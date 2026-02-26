mod pattern;
mod types;
#[allow(unused_imports)]
pub use super::error::HookError;
pub use pattern::ParsedPattern;
pub use types::{HookDefinition, HookOperation, PatternSegment, Phase};
#[cfg(test)]
#[path = "../config_tests_1.rs"]
mod config_tests_1;
#[cfg(test)]
#[path = "../config_tests_2.rs"]
mod config_tests_2;
#[cfg(test)]
#[path = "../config_tests_3.rs"]
mod config_tests_3;
