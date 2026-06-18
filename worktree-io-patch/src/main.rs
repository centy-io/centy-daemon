//! Worktree CLI binary — open GitHub issues as git worktree workspaces.
#![allow(missing_docs)] // binary crate; public API lives in the library
use anyhow::Result;
use clap::{Parser, Subcommand};
mod commands;
use commands::config::{cmd_config, ConfigAction};
use commands::list::cmd_list;
use commands::open::cmd_open;
use commands::open_multi::cmd_open_multi;
use commands::prune::cmd_prune;
use commands::restore::cmd_restore;
use commands::scheme::{cmd_scheme, SchemeAction};
use commands::setup::cmd_setup;
#[derive(Parser)]
#[command(
    name = "worktree",
    about = "Open GitHub issues as git worktree workspaces",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Parse an issue reference, create a worktree, and open it
    Open {
        /// Issue reference (omit to detect from current repo's origin remote)
        #[arg(value_name = "REF")]
        issue_ref: Option<String>,
        /// Force open in editor
        #[arg(long)]
        editor: bool,
        /// Skip pre/post-open hooks
        #[arg(long)]
        no_hooks: bool,
        /// Skip opening editor/terminal (hooks still run); useful for programmatic invocation
        #[arg(long)]
        headless: bool,
    },
    /// Open multiple repos as a unified workspace under ~/workspaces/<name>/
    #[command(name = "open-multi")]
    OpenMulti {
        /// Issue references (owner/repo#N or full GitHub URL), at least two
        #[arg(value_name = "REF", num_args = 1..)]
        refs: Vec<String>,
        /// Skip pre/post-open hooks
        #[arg(long)]
        no_hooks: bool,
    },
    /// Manage worktree configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Manage the worktree:// URL scheme handler
    Scheme {
        #[command(subcommand)]
        action: SchemeAction,
    },
    /// List all registered workspaces with their TTL status
    List {
        /// Emit a JSON report to stdout instead of human-readable output
        #[arg(long)]
        json: bool,
    },
    /// Remove expired worktrees based on workspace.ttl config
    Prune {
        /// Emit a JSON report to stdout instead of human-readable output
        #[arg(long)]
        json: bool,
    },
    /// Restore worktrees whose directories were manually deleted
    Restore,
    /// Run first-time setup: detect editor, write config, register URL scheme
    Setup,
    /// Print the current version
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Open {
            issue_ref,
            editor,
            no_hooks,
            headless,
        } => cmd_open(issue_ref.as_deref(), editor, no_hooks, headless)?,
        Commands::OpenMulti { refs, no_hooks } => cmd_open_multi(&refs, no_hooks)?,
        Commands::Config { action } => cmd_config(action)?,
        Commands::List { json } => cmd_list(json)?,
        Commands::Prune { json } => cmd_prune(json)?,
        Commands::Restore => cmd_restore()?,
        Commands::Scheme { action } => cmd_scheme(action)?,
        Commands::Setup => cmd_setup()?,
        Commands::Version => println!("{}", env!("CARGO_PKG_VERSION")),
    }
    Ok(())
}
