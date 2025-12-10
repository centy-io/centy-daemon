pub mod agent;
pub mod config;
pub mod prompt;
pub mod work;

pub use agent::{check_agent_available, get_available_agents, spawn_agent, AgentError, SpawnResult};
pub use config::{
    get_effective_local_config, get_global_centy_config_dir, has_global_config, has_project_config,
    read_global_local_config, read_project_local_config, write_global_local_config,
    write_project_local_config, AgentConfig, AgentType, LocalConfigError, LocalLlmConfig,
};
pub use prompt::{
    LlmAction, PromptBuilder, PromptError, BASE_SYSTEM_PROMPT, IMPLEMENT_ACTION_PROMPT,
    PLAN_ACTION_PROMPT,
};
pub use work::{
    clear_work_session, get_active_work_status, is_process_running, read_work_session,
    record_work_session, LlmWorkSession, WorkTrackingError,
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("Config error: {0}")]
    ConfigError(#[from] LocalConfigError),

    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),

    #[error("Prompt error: {0}")]
    PromptError(#[from] PromptError),

    #[error("Work tracking error: {0}")]
    WorkTrackingError(#[from] WorkTrackingError),

    #[error("Issue error: {0}")]
    IssueError(#[from] crate::issue::IssueCrudError),
}
