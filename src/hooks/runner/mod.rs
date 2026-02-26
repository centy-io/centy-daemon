mod common;
mod post_hooks;
mod pre_hooks;
#[allow(unused_imports)]
pub use super::config::{HookDefinition, HookOperation, Phase};
#[allow(unused_imports)]
pub use common::find_matching_hooks;
pub use post_hooks::run_post_hooks;
pub use pre_hooks::run_pre_hooks;
#[cfg(test)]
#[path = "../runner_tests_1.rs"]
mod runner_tests_1;
#[cfg(test)]
#[path = "../runner_tests_2.rs"]
mod runner_tests_2;
