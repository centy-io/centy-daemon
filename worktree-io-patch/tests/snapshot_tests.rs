//! Snapshot tests for generated file contents and paths.
//!
//! These tests verify that the files written by `worktree-io` contain exactly
//! the expected content and land at the expected paths.  Running them inside a
//! clean Docker container (see `docker/Dockerfile.test`) eliminates host-state
//! pollution and keeps the snapshots deterministic.
#![allow(missing_docs)]

use worktree_io::repo_hooks_scaffold::SCAFFOLD;
use worktree_io::{config::Config, ttl::WorkspaceRegistry};

// ── .worktree.toml ────────────────────────────────────────────────────────────

/// The scaffold written to `.worktree.toml` when a worktree is first opened.
#[test]
fn test_worktree_toml_scaffold_content() {
    insta::assert_snapshot!(SCAFFOLD);
}

// ── config.toml ───────────────────────────────────────────────────────────────

/// Default `config.toml` written by `worktree config init`.
#[test]
fn test_config_toml_default_content() {
    insta::assert_snapshot!(Config::default().to_toml_with_comments());
}

/// `config.toml` with every optional field populated.
#[test]
fn test_config_toml_all_fields() {
    let mut c = Config::default();
    c.editor.command = Some("code .".into());
    c.open.editor = false;
    c.hooks.pre_open = Some("cargo build".into());
    c.hooks.post_open = Some("npm install".into());
    c.set_value("workspace.ttl", "7days").unwrap();
    c.set_value("workspace.auto_prune", "true").unwrap();
    insta::assert_snapshot!(c.to_toml_with_comments());
}

/// Path suffix of `config.toml` relative to `$HOME` — stable across hosts.
#[test]
fn test_config_toml_path_suffix() {
    let home = dirs::home_dir().expect("home dir must exist");
    let path = Config::path().expect("Config::path() must succeed");
    let suffix = path
        .strip_prefix(&home)
        .expect("config path must be under $HOME");
    insta::assert_snapshot!(suffix.to_str().unwrap());
}

// ── workspaces.toml ───────────────────────────────────────────────────────────

/// Path suffix of `workspaces.toml` relative to `$HOME` — stable across hosts.
#[test]
fn test_workspaces_toml_path_suffix() {
    let home = dirs::home_dir().expect("home dir must exist");
    let path = WorkspaceRegistry::path().expect("WorkspaceRegistry::path() must succeed");
    let suffix = path
        .strip_prefix(&home)
        .expect("registry path must be under $HOME");
    insta::assert_snapshot!(suffix.to_str().unwrap());
}

/// A freshly serialized empty registry round-trips through TOML cleanly.
#[test]
fn test_workspaces_toml_empty_content() {
    let registry = WorkspaceRegistry::default();
    let toml = toml::to_string(&registry).expect("serialization must succeed");
    insta::assert_snapshot!(toml);
}
