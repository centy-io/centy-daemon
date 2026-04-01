mod pattern;
mod types;
pub use super::error::HookError;
pub use pattern::ParsedPattern;
pub use types::{HookDefinition, HookOperation, HooksFile, PatternSegment, Phase};
#[cfg(test)]
#[path = "../hook_pattern_parsing_tests.rs"]
mod hook_pattern_parsing_tests;
#[cfg(test)]
#[path = "../hook_pattern_segment_matching.rs"]
mod hook_pattern_segment_matching;
#[cfg(test)]
#[path = "../hook_phase_and_operation.rs"]
mod hook_phase_and_operation;
