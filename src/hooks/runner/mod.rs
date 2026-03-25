mod common;
mod post_hooks;
mod pre_hooks;
pub use super::config::{HookDefinition, HookOperation, Phase};
pub use common::find_matching_hooks;
pub use post_hooks::run_post_hooks;
pub use pre_hooks::run_pre_hooks;
#[cfg(test)]
#[path = "../find_matching_hooks.rs"]
mod find_matching_hooks;
#[cfg(test)]
#[path = "../hook_specificity_ordering.rs"]
mod hook_specificity_ordering;
