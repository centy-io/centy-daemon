// Allow unknown/renamed lints (shared with lib.rs for consistency across targets)
#![allow(unknown_lints, renamed_and_removed_lints)]
// Allow no_wrapper_functions dylint: many legitimate helper functions are flagged as wrappers
#![allow(no_wrapper_functions)]
// Allow unused imports/dead_code for pub use re-exports that form the lib's public API but aren't all used by the binary
#![allow(unused_imports, dead_code)]
// Allow panic/unwrap/expect in tests (denied globally via Cargo.toml lints)
#![cfg_attr(
    test,
    allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic_in_result_fn,
        clippy::unwrap_in_result,
        clippy::arithmetic_side_effects,
        clippy::indexing_slicing,
        clippy::wildcard_imports,
        clippy::field_reassign_with_default
    )
)]

mod app;
mod cleanup;
mod common;
mod config;
mod cors;
mod hooks;
mod item;
mod link;
mod logging;
mod manifest;
mod metrics;
mod reconciliation;
mod registry;
mod run;
mod server;
mod template;
mod user;
mod user_config;
mod utils;
mod workspace;

use clap::Parser as _;
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(color_eyre::eyre::Report::from)?
        .block_on(run::run(app::Args::parse()))
}
