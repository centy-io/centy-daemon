mod common;
mod post_hooks;
mod pre_hooks;
pub use super::config::{HookDefinition, HookOperation, Phase};
pub use common::find_matching_hooks;
pub use common::load_hooks_config;
pub use post_hooks::run_post_hooks;
pub use pre_hooks::run_pre_hooks;
#[cfg(test)]
#[path = "../find_matching_hooks_tests.rs"]
mod find_matching_hooks_tests;
#[cfg(test)]
#[path = "../hook_specificity_ordering_tests.rs"]
mod hook_specificity_ordering_tests;
