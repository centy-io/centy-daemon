use super::super::error::HookError;
use super::types::{HookOperation, PatternSegment, Phase};
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
        let phase_str = parts.first().ok_or_else(|| {
            HookError::InvalidPattern(format!("Missing phase segment in pattern: '{pattern}'"))
        })?;
        let item_type_str = parts.get(1).ok_or_else(|| {
            HookError::InvalidPattern(format!("Missing item_type segment in pattern: '{pattern}'"))
        })?;
        let operation_str = parts.get(2).ok_or_else(|| {
            HookError::InvalidPattern(format!("Missing operation segment in pattern: '{pattern}'"))
        })?;
        let phase = Self::parse_segment(phase_str, &["pre", "post"])?;
        let item_type = Self::parse_item_type_segment(item_type_str)?;
        let operation = Self::parse_segment(
            operation_str,
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
    fn parse_item_type_segment(value: &str) -> Result<PatternSegment, HookError> {
        if value == "*" {
            return Ok(PatternSegment::Wildcard);
        }
        if value.is_empty() {
            return Err(HookError::InvalidPattern(
                "Item type segment must not be empty".to_string(),
            ));
        }
        Ok(PatternSegment::Exact(value.to_string()))
    }
    pub fn matches(&self, phase: Phase, item_type: &str, operation: HookOperation) -> bool {
        Self::segment_matches(&self.phase, phase.as_str())
            && Self::segment_matches(&self.item_type, item_type)
            && Self::segment_matches(&self.operation, operation.as_str())
    }
    fn segment_matches(segment: &PatternSegment, value: &str) -> bool {
        match segment {
            PatternSegment::Wildcard => true,
            PatternSegment::Exact(s) => s == value,
        }
    }
    pub fn specificity(&self) -> u8 {
        let mut count: u8 = 0;
        if self.phase != PatternSegment::Wildcard {
            count = count.saturating_add(1);
        }
        if self.item_type != PatternSegment::Wildcard {
            count = count.saturating_add(1);
        }
        if self.operation != PatternSegment::Wildcard {
            count = count.saturating_add(1);
        }
        count
    }
}
