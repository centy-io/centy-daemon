/// Return only the editors/terminals available on the current system.
pub mod available_entries;
mod editor;
/// Unified table of all supported editors and terminals.
pub mod entries;
mod is_available;
mod shell;
mod terminal;
#[cfg(windows)]
mod wt;

pub use editor::resolve_editor_command;
pub use shell::augmented_path;

use anyhow::{Context, Result};
use std::path::Path;

/// Check whether a macOS application bundle is installed.
#[cfg(target_os = "macos")]
fn app_exists(name: &str) -> bool {
    std::path::Path::new(&format!("/Applications/{name}.app")).exists()
        || std::path::Path::new(&format!("/System/Applications/{name}.app")).exists()
}

/// For the IDE case: find an available terminal app and run `init_script` inside it.
fn open_hook_in_auto_terminal(path: &Path, init_script: &str) -> Result<bool> {
    #[cfg(windows)]
    if which::which("wt").is_ok() && terminal::try_terminal_with_init(path, "wt", init_script)? {
        return Ok(true);
    }
    if which::which("tmux").is_ok() && terminal::try_terminal_with_init(path, "tmux", init_script)?
    {
        return Ok(true);
    }
    #[cfg(target_os = "macos")]
    {
        let candidates: &[(&str, &str)] = &[
            ("iTerm", "open -a iTerm ."),
            ("Warp", "open -a Warp ."),
            ("Ghostty", "open -a Ghostty ."),
            ("Terminal", "open -a Terminal ."),
        ];
        for &(app, cmd) in candidates {
            // LLVM_COV_EXCL_START
            if app_exists(app) && terminal::try_terminal_with_init(path, cmd, init_script)? {
                return Ok(true);
            }
            // LLVM_COV_EXCL_STOP
        }
    }
    Ok(false)
}

/// Open `path` with `command` and run `init_script` inside the resulting window.
///
/// # Errors
///
/// Returns an error if the editor or terminal command fails to spawn.
pub fn open_with_hook(path: &Path, cmd: &str, init: &str, background: bool) -> Result<bool> {
    if terminal::try_terminal_with_init(path, cmd, init)? {
        return Ok(true);
    }
    open_in_editor(path, cmd, background)?;
    open_hook_in_auto_terminal(path, init)
}

/// Open the workspace path in the configured editor.
///
/// # Errors
///
/// Returns an error if the workspace path is not valid UTF-8 or the editor
/// command fails to spawn.
pub fn open_in_editor(path: &Path, command: &str, background: bool) -> Result<()> {
    let path_str = path
        .to_str()
        .context("Workspace path contains non-UTF-8 characters")?;
    let cmd_str = if command.contains(" . ") || command.ends_with(" .") || command == "." {
        command.replacen(" .", &format!(" {path_str}"), 1)
    } else {
        format!("{command} {path_str}")
    };

    shell::run_shell_command(&cmd_str, background)
        .with_context(|| format!("Failed to open editor with command: {cmd_str}"))
}

/// Open `path` with `cmd`; uses terminal-specific logic if `cmd` is a known terminal,
/// otherwise opens as an editor.
/// # Errors
/// Returns an error if the spawn or editor command fails.
pub fn open_editor_or_terminal(path: &Path, cmd: &str, background: bool) -> Result<()> {
    if !terminal::try_terminal_with_init(path, cmd, "")? {
        open_in_editor(path, cmd, background)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "opener_tests.rs"]
mod tests;
