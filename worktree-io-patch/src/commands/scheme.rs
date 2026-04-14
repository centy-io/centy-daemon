use anyhow::Result;
use clap::Subcommand;
use worktree_io::scheme;

#[derive(Copy, Clone, Subcommand)]
pub enum SchemeAction {
    /// Unregister the worktree:// URL scheme handler
    Uninstall,
    /// Check whether the URL scheme handler is registered
    Status,
}

pub fn cmd_scheme(action: SchemeAction) -> Result<()> {
    // LLVM_COV_EXCL_LINE
    // LLVM_COV_EXCL_START
    match action {
        SchemeAction::Uninstall => scheme::uninstall()?,
        SchemeAction::Status => println!("{}", scheme::status()?),
    }
    Ok(())
    // LLVM_COV_EXCL_STOP
}
