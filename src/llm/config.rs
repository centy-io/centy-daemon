use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;

use crate::utils::get_centy_path;

#[derive(Error, Debug)]
pub enum LocalConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Home directory not found")]
    HomeDirNotFound,
}

/// Predefined agent types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum AgentType {
    #[default]
    Claude,
    Gemini,
    Codex,
    Opencode,
    Custom,
}


/// Configuration for a single agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    pub agent_type: AgentType,
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub default_args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implement_template: Option<String>,
}

/// Local LLM configuration (not version controlled)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LocalLlmConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_agent: Option<String>,
    #[serde(default)]
    pub agents: Vec<AgentConfig>,
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
}

impl LocalLlmConfig {
    /// Create default config with predefined agents
    #[must_use] 
    pub fn with_defaults() -> Self {
        Self {
            default_agent: Some("claude".to_string()),
            agents: vec![
                AgentConfig {
                    agent_type: AgentType::Claude,
                    name: "claude".to_string(),
                    command: "claude".to_string(),
                    default_args: vec!["--print".to_string()],
                    plan_template: None,
                    implement_template: None,
                },
                AgentConfig {
                    agent_type: AgentType::Gemini,
                    name: "gemini".to_string(),
                    command: "gemini".to_string(),
                    default_args: vec![],
                    plan_template: None,
                    implement_template: None,
                },
                AgentConfig {
                    agent_type: AgentType::Codex,
                    name: "codex".to_string(),
                    command: "codex".to_string(),
                    default_args: vec![],
                    plan_template: None,
                    implement_template: None,
                },
                AgentConfig {
                    agent_type: AgentType::Opencode,
                    name: "opencode".to_string(),
                    command: "opencode".to_string(),
                    default_args: vec![],
                    plan_template: None,
                    implement_template: None,
                },
            ],
            env_vars: HashMap::new(),
        }
    }

    /// Merge project config over global config
    #[must_use] 
    pub fn merge(global: Self, project: Self) -> Self {
        let mut merged = global;

        // Project default_agent overrides global
        if project.default_agent.is_some() {
            merged.default_agent = project.default_agent;
        }

        // Project agents override/add to global agents
        for agent in project.agents {
            if let Some(existing) = merged.agents.iter_mut().find(|a| a.name == agent.name) {
                *existing = agent;
            } else {
                merged.agents.push(agent);
            }
        }

        // Project env_vars override global
        merged.env_vars.extend(project.env_vars);

        merged
    }

    /// Get agent by name
    #[must_use] 
    pub fn get_agent(&self, name: &str) -> Option<&AgentConfig> {
        self.agents.iter().find(|a| a.name == name)
    }

    /// Get default agent
    #[must_use] 
    pub fn get_default_agent(&self) -> Option<&AgentConfig> {
        self.default_agent
            .as_ref()
            .and_then(|name| self.get_agent(name))
    }
}

const LOCAL_CONFIG_FILE: &str = "config.local.json";

/// Get the path to the global centy config directory (~/.centy)
pub fn get_global_centy_config_dir() -> Result<PathBuf, LocalConfigError> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| LocalConfigError::HomeDirNotFound)?;

    Ok(PathBuf::from(home).join(".centy"))
}

/// Read global local config (~/.centy/config.local.json)
pub async fn read_global_local_config() -> Result<Option<LocalLlmConfig>, LocalConfigError> {
    let config_dir = get_global_centy_config_dir()?;
    let path = config_dir.join(LOCAL_CONFIG_FILE);

    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path).await?;
    let config: LocalLlmConfig = serde_json::from_str(&content)?;
    Ok(Some(config))
}

/// Read project-specific local config (.centy/config.local.json)
pub async fn read_project_local_config(
    project_path: &Path,
) -> Result<Option<LocalLlmConfig>, LocalConfigError> {
    let path = get_centy_path(project_path).join(LOCAL_CONFIG_FILE);

    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path).await?;
    let config: LocalLlmConfig = serde_json::from_str(&content)?;
    Ok(Some(config))
}

/// Get effective local config (project + global merged, with defaults)
pub async fn get_effective_local_config(
    project_path: Option<&Path>,
) -> Result<LocalLlmConfig, LocalConfigError> {
    let global = read_global_local_config().await?.unwrap_or_default();

    let effective = if let Some(path) = project_path {
        let project = read_project_local_config(path).await?.unwrap_or_default();
        LocalLlmConfig::merge(global, project)
    } else {
        global
    };

    // Ensure we always have the predefined agents as a base
    let defaults = LocalLlmConfig::with_defaults();
    Ok(LocalLlmConfig::merge(defaults, effective))
}

/// Write global local config
pub async fn write_global_local_config(config: &LocalLlmConfig) -> Result<(), LocalConfigError> {
    let config_dir = get_global_centy_config_dir()?;
    fs::create_dir_all(&config_dir).await?;

    let path = config_dir.join(LOCAL_CONFIG_FILE);
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&path, content).await?;
    Ok(())
}

/// Write project-specific local config
pub async fn write_project_local_config(
    project_path: &Path,
    config: &LocalLlmConfig,
) -> Result<(), LocalConfigError> {
    let path = get_centy_path(project_path).join(LOCAL_CONFIG_FILE);
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&path, content).await?;
    Ok(())
}

/// Check if global config exists
pub async fn has_global_config() -> bool {
    if let Ok(dir) = get_global_centy_config_dir() {
        dir.join(LOCAL_CONFIG_FILE).exists()
    } else {
        false
    }
}

/// Check if project config exists
#[must_use] 
pub fn has_project_config(project_path: &Path) -> bool {
    get_centy_path(project_path).join(LOCAL_CONFIG_FILE).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_serialization() {
        let agent_type = AgentType::Claude;
        let json = serde_json::to_string(&agent_type).unwrap();
        assert_eq!(json, "\"claude\"");

        let deserialized: AgentType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, AgentType::Claude);
    }

    #[test]
    fn test_local_config_defaults() {
        let config = LocalLlmConfig::with_defaults();
        assert_eq!(config.default_agent, Some("claude".to_string()));
        assert_eq!(config.agents.len(), 4);

        let claude = config.get_agent("claude").unwrap();
        assert_eq!(claude.agent_type, AgentType::Claude);
        assert_eq!(claude.command, "claude");
    }

    #[test]
    fn test_config_merge() {
        let global = LocalLlmConfig {
            default_agent: Some("claude".to_string()),
            agents: vec![AgentConfig {
                agent_type: AgentType::Claude,
                name: "claude".to_string(),
                command: "claude".to_string(),
                default_args: vec!["--global".to_string()],
                plan_template: None,
                implement_template: None,
            }],
            env_vars: HashMap::from([("GLOBAL_KEY".to_string(), "global_value".to_string())]),
        };

        let project = LocalLlmConfig {
            default_agent: Some("gemini".to_string()),
            agents: vec![AgentConfig {
                agent_type: AgentType::Claude,
                name: "claude".to_string(),
                command: "claude".to_string(),
                default_args: vec!["--project".to_string()],
                plan_template: Some("my-plan".to_string()),
                implement_template: None,
            }],
            env_vars: HashMap::from([("PROJECT_KEY".to_string(), "project_value".to_string())]),
        };

        let merged = LocalLlmConfig::merge(global, project);

        // Project default_agent should win
        assert_eq!(merged.default_agent, Some("gemini".to_string()));

        // Project agent config should override global
        let claude = merged.get_agent("claude").unwrap();
        assert_eq!(claude.default_args, vec!["--project".to_string()]);
        assert_eq!(claude.plan_template, Some("my-plan".to_string()));

        // Both env vars should be present
        assert_eq!(merged.env_vars.get("GLOBAL_KEY"), Some(&"global_value".to_string()));
        assert_eq!(merged.env_vars.get("PROJECT_KEY"), Some(&"project_value".to_string()));
    }

    #[test]
    fn test_get_default_agent() {
        let config = LocalLlmConfig::with_defaults();
        let default_agent = config.get_default_agent().unwrap();
        assert_eq!(default_agent.name, "claude");
    }

    #[test]
    fn test_agent_config_serialization() {
        let config = AgentConfig {
            agent_type: AgentType::Custom,
            name: "my-agent".to_string(),
            command: "/path/to/agent".to_string(),
            default_args: vec!["--flag".to_string()],
            plan_template: Some("plan-template".to_string()),
            implement_template: None,
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("\"agentType\": \"custom\""));
        assert!(json.contains("\"planTemplate\": \"plan-template\""));
        // implementTemplate should not be serialized (skip_serializing_if)
        assert!(!json.contains("implementTemplate"));

        let deserialized: AgentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "my-agent");
        assert_eq!(deserialized.implement_template, None);
    }
}
