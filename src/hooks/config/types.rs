use serde::{Deserialize, Serialize};
fn default_timeout() -> u64 { 30 }
fn default_enabled() -> bool { true }
/// Hook definition from config.json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookDefinition {
    pub pattern: String,
    pub command: String,
    #[serde(default, rename = "async")]
    pub is_async: bool,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}
/// Phase of hook execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase { Pre, Post }
impl Phase {
    pub fn as_str(&self) -> &'static str {
        match self { Phase::Pre => "pre", Phase::Post => "post" }
    }
}
/// Operations that hooks can target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookOperation { Create, Update, Delete, SoftDelete, Restore, Move, Duplicate }
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
pub enum PatternSegment { Exact(String), Wildcard }
