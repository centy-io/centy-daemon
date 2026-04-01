use super::super::error::HookError;
use super::types::{HookOperation, PatternSegment, Phase};
/// Valid event names for the event-driven pattern format
const VALID_EVENTS: &[&str] = &[
    "creating",
    "created",
    "updating",
    "updated",
    "deleting",
    "deleted",
    "soft-deleting",
    "soft-deleted",
    "restoring",
    "restored",
    "moving",
    "moved",
    "duplicating",
    "duplicated",
];
/// Maps a (phase, operation) pair to its event name string.
fn event_name(phase: Phase, operation: HookOperation) -> &'static str {
    match (phase, operation) {
        (Phase::Pre, HookOperation::Create) => "creating",
        (Phase::Post, HookOperation::Create) => "created",
        (Phase::Pre, HookOperation::Update) => "updating",
        (Phase::Post, HookOperation::Update) => "updated",
        (Phase::Pre, HookOperation::Delete) => "deleting",
        (Phase::Post, HookOperation::Delete) => "deleted",
        (Phase::Pre, HookOperation::SoftDelete) => "soft-deleting",
        (Phase::Post, HookOperation::SoftDelete) => "soft-deleted",
        (Phase::Pre, HookOperation::Restore) => "restoring",
        (Phase::Post, HookOperation::Restore) => "restored",
        (Phase::Pre, HookOperation::Move) => "moving",
        (Phase::Post, HookOperation::Move) => "moved",
        (Phase::Pre, HookOperation::Duplicate) => "duplicating",
        (Phase::Post, HookOperation::Duplicate) => "duplicated",
    }
}
/// A parsed hook pattern (e.g., "issue.creating" or "*.deleted")
#[derive(Debug, Clone)]
pub struct ParsedPattern {
    pub item_type: PatternSegment,
    pub event: PatternSegment,
}
impl ParsedPattern {
    /// Parse a pattern string like "issue.creating" or "*.deleted" or "*.*"
    pub fn parse(pattern: &str) -> Result<Self, HookError> {
        let Some((item_type_str, event_str)) = pattern.split_once('.') else {
            return Err(HookError::InvalidPattern(format!(
                "Pattern must have exactly 2 segments (item_type.event), got: '{pattern}'"
            )));
        };
        let item_type = Self::parse_item_type_segment(item_type_str, pattern)?;
        let event = Self::parse_event_segment(event_str, pattern)?;
        Ok(ParsedPattern { item_type, event })
    }
    fn parse_item_type_segment(value: &str, pattern: &str) -> Result<PatternSegment, HookError> {
        if value == "*" {
            return Ok(PatternSegment::Wildcard);
        }
        if value.is_empty() {
            return Err(HookError::InvalidPattern(format!(
                "Item type segment must not be empty in pattern: '{pattern}'"
            )));
        }
        Ok(PatternSegment::Exact(value.to_string()))
    }
    fn parse_event_segment(value: &str, pattern: &str) -> Result<PatternSegment, HookError> {
        if value == "*" {
            return Ok(PatternSegment::Wildcard);
        }
        if VALID_EVENTS.contains(&value) {
            return Ok(PatternSegment::Exact(value.to_string()));
        }
        Err(HookError::InvalidPattern(format!(
            "Invalid event '{value}' in pattern '{pattern}', expected one of: {}, or '*'",
            VALID_EVENTS.join(", ")
        )))
    }
    #[must_use]
    pub fn matches(&self, phase: Phase, item_type: &str, operation: HookOperation) -> bool {
        let ev = event_name(phase, operation);
        Self::segment_matches(&self.item_type, item_type) && Self::segment_matches(&self.event, ev)
    }
    fn segment_matches(segment: &PatternSegment, value: &str) -> bool {
        match segment {
            PatternSegment::Wildcard => true,
            PatternSegment::Exact(s) => s == value,
        }
    }
    #[must_use]
    pub fn specificity(&self) -> u8 {
        let mut count: u8 = 0;
        if self.item_type != PatternSegment::Wildcard {
            count = count.saturating_add(1);
        }
        if self.event != PatternSegment::Wildcard {
            count = count.saturating_add(1);
        }
        count
    }
}
