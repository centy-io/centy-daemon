use serde::{Deserialize, Serialize};

use super::error::HookError;

/// Default timeout for hooks in seconds
fn default_timeout() -> u64 {
    30
}

/// Default enabled state for hooks
fn default_enabled() -> bool {
    true
}

/// Hook definition from config.json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookDefinition {
    /// Pattern like "pre:issue:create", "post:*:delete", "*:doc:*"
    pub pattern: String,
    /// Bash command to execute
    pub command: String,
    /// If true, run in background (post-hooks only)
    #[serde(default, rename = "async")]
    pub is_async: bool,
    /// Timeout in seconds (default 30)
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// Whether hook is enabled (default true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

/// Phase of hook execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Pre,
    Post,
}

impl Phase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Phase::Pre => "pre",
            Phase::Post => "post",
        }
    }
}

/// Item types that hooks can target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookItemType {
    Issue,
    Doc,
    Pr,
    User,
    Link,
    Asset,
}

impl HookItemType {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookItemType::Issue => "issue",
            HookItemType::Doc => "doc",
            HookItemType::Pr => "pr",
            HookItemType::User => "user",
            HookItemType::Link => "link",
            HookItemType::Asset => "asset",
        }
    }
}

/// Operations that hooks can target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookOperation {
    Create,
    Update,
    Delete,
    SoftDelete,
    Restore,
    Move,
    Duplicate,
}

impl HookOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookOperation::Create => "create",
            HookOperation::Update => "update",
            HookOperation::Delete => "delete",
            HookOperation::SoftDelete => "soft-delete",
            HookOperation::Restore => "restore",
            HookOperation::Move => "move",
            HookOperation::Duplicate => "duplicate",
        }
    }
}

/// A segment of a parsed pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternSegment {
    Exact(String),
    Wildcard,
}

/// A parsed hook pattern (e.g., "pre:issue:create")
#[derive(Debug, Clone)]
pub struct ParsedPattern {
    pub phase: PatternSegment,
    pub item_type: PatternSegment,
    pub operation: PatternSegment,
}

impl ParsedPattern {
    /// Parse a pattern string like "pre:issue:create" or "*:*:delete"
    pub fn parse(pattern: &str) -> Result<Self, HookError> {
        let parts: Vec<&str> = pattern.split(':').collect();
        if parts.len() != 3 {
            return Err(HookError::InvalidPattern(format!(
                "Pattern must have exactly 3 segments (phase:item_type:operation), got: '{pattern}'"
            )));
        }

        let phase = Self::parse_segment(parts[0], &["pre", "post"])?;
        let item_type =
            Self::parse_segment(parts[1], &["issue", "doc", "pr", "user", "link", "asset"])?;
        let operation = Self::parse_segment(
            parts[2],
            &[
                "create",
                "update",
                "delete",
                "soft-delete",
                "restore",
                "move",
                "duplicate",
            ],
        )?;

        Ok(ParsedPattern {
            phase,
            item_type,
            operation,
        })
    }

    fn parse_segment(value: &str, valid_values: &[&str]) -> Result<PatternSegment, HookError> {
        if value == "*" {
            return Ok(PatternSegment::Wildcard);
        }
        if valid_values.contains(&value) {
            return Ok(PatternSegment::Exact(value.to_string()));
        }
        Err(HookError::InvalidPattern(format!(
            "Invalid pattern value '{value}', expected one of: {}, or '*'",
            valid_values.join(", ")
        )))
    }

    /// Check if this pattern matches a given phase, item_type, and operation
    pub fn matches(&self, phase: Phase, item_type: HookItemType, operation: HookOperation) -> bool {
        Self::segment_matches(&self.phase, phase.as_str())
            && Self::segment_matches(&self.item_type, item_type.as_str())
            && Self::segment_matches(&self.operation, operation.as_str())
    }

    fn segment_matches(segment: &PatternSegment, value: &str) -> bool {
        match segment {
            PatternSegment::Wildcard => true,
            PatternSegment::Exact(s) => s == value,
        }
    }

    /// Count of non-wildcard segments (0-3). Higher = more specific.
    pub fn specificity(&self) -> u8 {
        let mut count = 0;
        if self.phase != PatternSegment::Wildcard {
            count += 1;
        }
        if self.item_type != PatternSegment::Wildcard {
            count += 1;
        }
        if self.operation != PatternSegment::Wildcard {
            count += 1;
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_pattern() {
        let p = ParsedPattern::parse("pre:issue:create").unwrap();
        assert_eq!(p.phase, PatternSegment::Exact("pre".to_string()));
        assert_eq!(p.item_type, PatternSegment::Exact("issue".to_string()));
        assert_eq!(p.operation, PatternSegment::Exact("create".to_string()));
    }

    #[test]
    fn test_parse_wildcard_pattern() {
        let p = ParsedPattern::parse("*:*:delete").unwrap();
        assert_eq!(p.phase, PatternSegment::Wildcard);
        assert_eq!(p.item_type, PatternSegment::Wildcard);
        assert_eq!(p.operation, PatternSegment::Exact("delete".to_string()));
    }

    #[test]
    fn test_parse_all_wildcards() {
        let p = ParsedPattern::parse("*:*:*").unwrap();
        assert_eq!(p.phase, PatternSegment::Wildcard);
        assert_eq!(p.item_type, PatternSegment::Wildcard);
        assert_eq!(p.operation, PatternSegment::Wildcard);
    }

    #[test]
    fn test_parse_invalid_segment_count() {
        let err = ParsedPattern::parse("pre:issue").unwrap_err();
        assert!(matches!(err, HookError::InvalidPattern(_)));
    }

    #[test]
    fn test_parse_invalid_phase() {
        let err = ParsedPattern::parse("during:issue:create").unwrap_err();
        assert!(matches!(err, HookError::InvalidPattern(_)));
    }

    #[test]
    fn test_parse_invalid_item_type() {
        let err = ParsedPattern::parse("pre:widget:create").unwrap_err();
        assert!(matches!(err, HookError::InvalidPattern(_)));
    }

    #[test]
    fn test_parse_invalid_operation() {
        let err = ParsedPattern::parse("pre:issue:explode").unwrap_err();
        assert!(matches!(err, HookError::InvalidPattern(_)));
    }

    #[test]
    fn test_matches_exact() {
        let p = ParsedPattern::parse("pre:issue:create").unwrap();
        assert!(p.matches(Phase::Pre, HookItemType::Issue, HookOperation::Create));
        assert!(!p.matches(Phase::Post, HookItemType::Issue, HookOperation::Create));
        assert!(!p.matches(Phase::Pre, HookItemType::Doc, HookOperation::Create));
        assert!(!p.matches(Phase::Pre, HookItemType::Issue, HookOperation::Delete));
    }

    #[test]
    fn test_matches_wildcard_phase() {
        let p = ParsedPattern::parse("*:issue:create").unwrap();
        assert!(p.matches(Phase::Pre, HookItemType::Issue, HookOperation::Create));
        assert!(p.matches(Phase::Post, HookItemType::Issue, HookOperation::Create));
        assert!(!p.matches(Phase::Pre, HookItemType::Doc, HookOperation::Create));
    }

    #[test]
    fn test_matches_wildcard_item_type() {
        let p = ParsedPattern::parse("pre:*:create").unwrap();
        assert!(p.matches(Phase::Pre, HookItemType::Issue, HookOperation::Create));
        assert!(p.matches(Phase::Pre, HookItemType::Doc, HookOperation::Create));
        assert!(p.matches(Phase::Pre, HookItemType::Pr, HookOperation::Create));
        assert!(!p.matches(Phase::Post, HookItemType::Issue, HookOperation::Create));
    }

    #[test]
    fn test_matches_all_wildcards() {
        let p = ParsedPattern::parse("*:*:*").unwrap();
        assert!(p.matches(Phase::Pre, HookItemType::Issue, HookOperation::Create));
        assert!(p.matches(Phase::Post, HookItemType::Doc, HookOperation::Delete));
    }

    #[test]
    fn test_specificity() {
        assert_eq!(ParsedPattern::parse("*:*:*").unwrap().specificity(), 0);
        assert_eq!(ParsedPattern::parse("pre:*:*").unwrap().specificity(), 1);
        assert_eq!(
            ParsedPattern::parse("pre:issue:*").unwrap().specificity(),
            2
        );
        assert_eq!(
            ParsedPattern::parse("pre:issue:create")
                .unwrap()
                .specificity(),
            3
        );
        assert_eq!(ParsedPattern::parse("*:*:delete").unwrap().specificity(), 1);
        assert_eq!(
            ParsedPattern::parse("*:issue:delete")
                .unwrap()
                .specificity(),
            2
        );
    }

    #[test]
    fn test_soft_delete_pattern() {
        let p = ParsedPattern::parse("post:issue:soft-delete").unwrap();
        assert!(p.matches(Phase::Post, HookItemType::Issue, HookOperation::SoftDelete));
        assert!(!p.matches(Phase::Post, HookItemType::Issue, HookOperation::Delete));
    }

    #[test]
    fn test_all_item_types() {
        let p = ParsedPattern::parse("pre:asset:create").unwrap();
        assert!(p.matches(Phase::Pre, HookItemType::Asset, HookOperation::Create));

        let p = ParsedPattern::parse("pre:link:create").unwrap();
        assert!(p.matches(Phase::Pre, HookItemType::Link, HookOperation::Create));

        let p = ParsedPattern::parse("pre:user:create").unwrap();
        assert!(p.matches(Phase::Pre, HookItemType::User, HookOperation::Create));

        let p = ParsedPattern::parse("pre:pr:create").unwrap();
        assert!(p.matches(Phase::Pre, HookItemType::Pr, HookOperation::Create));
    }

    // --- Phase tests ---

    #[test]
    fn test_phase_as_str() {
        assert_eq!(Phase::Pre.as_str(), "pre");
        assert_eq!(Phase::Post.as_str(), "post");
    }

    #[test]
    fn test_phase_eq() {
        assert_eq!(Phase::Pre, Phase::Pre);
        assert_eq!(Phase::Post, Phase::Post);
        assert_ne!(Phase::Pre, Phase::Post);
    }

    // --- HookItemType tests ---

    #[test]
    fn test_hook_item_type_as_str() {
        assert_eq!(HookItemType::Issue.as_str(), "issue");
        assert_eq!(HookItemType::Doc.as_str(), "doc");
        assert_eq!(HookItemType::Pr.as_str(), "pr");
        assert_eq!(HookItemType::User.as_str(), "user");
        assert_eq!(HookItemType::Link.as_str(), "link");
        assert_eq!(HookItemType::Asset.as_str(), "asset");
    }

    // --- HookOperation tests ---

    #[test]
    fn test_hook_operation_as_str() {
        assert_eq!(HookOperation::Create.as_str(), "create");
        assert_eq!(HookOperation::Update.as_str(), "update");
        assert_eq!(HookOperation::Delete.as_str(), "delete");
        assert_eq!(HookOperation::SoftDelete.as_str(), "soft-delete");
        assert_eq!(HookOperation::Restore.as_str(), "restore");
        assert_eq!(HookOperation::Move.as_str(), "move");
        assert_eq!(HookOperation::Duplicate.as_str(), "duplicate");
    }

    // --- HookDefinition tests ---

    #[test]
    fn test_hook_definition_serialization() {
        let hook = HookDefinition {
            pattern: "pre:issue:create".to_string(),
            command: "echo hello".to_string(),
            is_async: false,
            timeout: 30,
            enabled: true,
        };

        let json = serde_json::to_string(&hook).expect("Should serialize");
        let deserialized: HookDefinition = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.pattern, "pre:issue:create");
        assert_eq!(deserialized.command, "echo hello");
        assert!(!deserialized.is_async);
        assert_eq!(deserialized.timeout, 30);
        assert!(deserialized.enabled);
    }

    #[test]
    fn test_hook_definition_defaults() {
        let json = r#"{"pattern":"pre:issue:create","command":"echo test"}"#;
        let hook: HookDefinition = serde_json::from_str(json).expect("Should deserialize");
        assert!(!hook.is_async);
        assert_eq!(hook.timeout, 30);
        assert!(hook.enabled);
    }

    #[test]
    fn test_hook_definition_camel_case() {
        let hook = HookDefinition {
            pattern: "test".to_string(),
            command: "cmd".to_string(),
            is_async: true,
            timeout: 60,
            enabled: false,
        };

        let json = serde_json::to_string(&hook).expect("Should serialize");
        assert!(json.contains("\"async\""));
        assert!(!json.contains("is_async"));
    }

    // --- PatternSegment tests ---

    #[test]
    fn test_pattern_segment_eq() {
        assert_eq!(PatternSegment::Wildcard, PatternSegment::Wildcard);
        assert_eq!(
            PatternSegment::Exact("pre".to_string()),
            PatternSegment::Exact("pre".to_string())
        );
        assert_ne!(
            PatternSegment::Wildcard,
            PatternSegment::Exact("*".to_string())
        );
    }

    // --- All operations patterns ---

    #[test]
    fn test_all_operations() {
        let ops = [
            ("create", HookOperation::Create),
            ("update", HookOperation::Update),
            ("delete", HookOperation::Delete),
            ("soft-delete", HookOperation::SoftDelete),
            ("restore", HookOperation::Restore),
            ("move", HookOperation::Move),
            ("duplicate", HookOperation::Duplicate),
        ];

        for (name, op) in &ops {
            let pattern = format!("pre:issue:{name}");
            let p = ParsedPattern::parse(&pattern)
                .unwrap_or_else(|_| panic!("Should parse pattern: {pattern}"));
            assert!(
                p.matches(Phase::Pre, HookItemType::Issue, *op),
                "Pattern '{pattern}' should match"
            );
        }
    }
}
