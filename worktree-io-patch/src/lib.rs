//! `worktree-io` — open GitHub, Linear, and local Centy issues as git worktree workspaces.

/// Configuration loading and serialization.
pub mod config;
/// Git operations: cloning, fetching, branch detection, and worktree creation.
pub mod git;
/// Pre/post-open hook execution.
pub mod hooks;
/// Issue reference types and parsing.
pub mod issue;
/// Multi-repo unified workspace creation.
pub mod multi_workspace;
/// Random human-friendly workspace name generator.
pub mod name_gen;
/// Editor and terminal openers.
pub mod opener;
/// Per-repository hook configuration loaded from `.worktree.toml`.
pub mod repo_hooks;
/// Scaffold template written to `.worktree.toml` when the file is absent.
pub mod repo_hooks_scaffold;
/// `worktree://` URL scheme registration.
pub mod scheme;
/// Compile-time TOML template assets embedded via `include_str!`.
pub mod templates;
/// Workspace TTL management and registry.
pub mod ttl;
/// Workspace creation and lifecycle.
pub mod workspace;

pub use config::Config;
pub use issue::{DeepLinkOptions, IssueRef};
pub use workspace::Workspace;
