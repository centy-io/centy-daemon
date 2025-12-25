//! Terminal integration for launching agents in terminal windows.
//!
//! Handles platform-specific terminal launching with agent commands.

use super::WorkspaceError;
use std::path::Path;
use std::process::Command;

/// Open a terminal with the specified agent command at the given working directory.
///
/// The terminal will:
/// 1. Change to the working directory
/// 2. Display an instruction message about the issue
/// 3. Execute the agent command
///
/// Returns Ok(true) if terminal was opened, Ok(false) if terminal is not available,
/// or Err if terminal failed to open.
pub fn open_terminal_with_agent(
    working_dir: &Path,
    display_number: u32,
    agent_command: &str,
    agent_args: &[String],
) -> Result<bool, WorkspaceError> {
    // Build the agent args string
    let args_str = if agent_args.is_empty() {
        String::new()
    } else {
        format!(" {}", agent_args.join(" "))
    };

    // Generate the shell script to run in terminal
    let shell_script = format!(
        r#"echo ""; echo "=== Centy Issue #{display_number} ==="; echo "Tip: Run 'centy get issue {display_number}' to view issue details"; echo ""; {agent_command}{args_str}"#
    );

    open_platform_terminal(working_dir, &shell_script)
}

/// Open a terminal at the specified directory with a command to run.
#[cfg(target_os = "macos")]
fn open_platform_terminal(working_dir: &Path, command: &str) -> Result<bool, WorkspaceError> {
    // Use osascript to open Terminal.app
    let script = format!(
        r#"tell application "Terminal"
    activate
    do script "cd '{}' && {}"
end tell"#,
        working_dir.display(),
        command.replace('\\', "\\\\").replace('"', "\\\"")
    );

    let result = Command::new("osascript").arg("-e").arg(&script).spawn();

    match result {
        Ok(_) => Ok(true),
        Err(e) => Err(WorkspaceError::TerminalError(e.to_string())),
    }
}

#[cfg(target_os = "linux")]
fn open_platform_terminal(working_dir: &Path, command: &str) -> Result<bool, WorkspaceError> {
    // Try common terminal emulators in order of preference
    let working_dir_str = working_dir.to_string_lossy();

    // Try gnome-terminal
    if which::which("gnome-terminal").is_ok() {
        let result = Command::new("gnome-terminal")
            .arg("--working-directory")
            .arg(&*working_dir_str)
            .arg("--")
            .arg("bash")
            .arg("-c")
            .arg(format!("{command}; exec bash"))
            .spawn();

        if result.is_ok() {
            return Ok(true);
        }
    }

    // Try konsole
    if which::which("konsole").is_ok() {
        let result = Command::new("konsole")
            .arg("--workdir")
            .arg(&*working_dir_str)
            .arg("-e")
            .arg("bash")
            .arg("-c")
            .arg(format!("{command}; exec bash"))
            .spawn();

        if result.is_ok() {
            return Ok(true);
        }
    }

    // Try xterm as fallback
    if which::which("xterm").is_ok() {
        let result = Command::new("xterm")
            .arg("-e")
            .arg(format!(
                "cd '{}' && {}; exec bash",
                working_dir.display(),
                command
            ))
            .spawn();

        if result.is_ok() {
            return Ok(true);
        }
    }

    Err(WorkspaceError::TerminalNotFound)
}

#[cfg(target_os = "windows")]
fn open_platform_terminal(working_dir: &Path, command: &str) -> Result<bool, WorkspaceError> {
    // Try Windows Terminal first
    let wt_result = Command::new("wt")
        .arg("-d")
        .arg(working_dir)
        .arg("cmd")
        .arg("/k")
        .arg(command)
        .spawn();

    if wt_result.is_ok() {
        return Ok(true);
    }

    // Fallback to cmd.exe
    let cmd_script = format!(
        "cd /d \"{}\" && {}",
        working_dir.display(),
        command
    );

    let result = Command::new("cmd")
        .arg("/k")
        .arg(&cmd_script)
        .spawn();

    match result {
        Ok(_) => Ok(true),
        Err(e) => Err(WorkspaceError::TerminalError(e.to_string())),
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn open_platform_terminal(_working_dir: &Path, _command: &str) -> Result<bool, WorkspaceError> {
    Err(WorkspaceError::TerminalNotFound)
}

/// Check if a terminal is available on the current platform.
#[must_use]
pub fn is_terminal_available() -> bool {
    #[cfg(target_os = "macos")]
    {
        // Terminal.app is always available on macOS
        true
    }

    #[cfg(target_os = "linux")]
    {
        which::which("gnome-terminal").is_ok()
            || which::which("konsole").is_ok()
            || which::which("xterm").is_ok()
    }

    #[cfg(target_os = "windows")]
    {
        // cmd.exe is always available on Windows
        true
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_terminal_available_returns_bool() {
        // Just verify it returns without panicking
        let _ = is_terminal_available();
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_terminal_always_available() {
        assert!(is_terminal_available());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_terminal_always_available() {
        assert!(is_terminal_available());
    }
}
