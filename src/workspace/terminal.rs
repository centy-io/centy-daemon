//! Terminal integration for opening terminal windows.
//!
//! Handles platform-specific terminal launching.

use super::WorkspaceError;
use std::path::Path;
use std::process::Command;

/// Escape a path string for safe use in shell commands.
///
/// On Unix, uses the `shell_escape` crate for POSIX-compatible escaping.
/// On Windows, wraps in double quotes (double-quote chars are stripped since
/// they are invalid in Windows filenames).
pub(crate) fn escape_path_for_shell(path: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        // Double quotes are invalid in Windows filenames, but strip them defensively
        format!("\"{}\"", path.replace('"', ""))
    }
    #[cfg(not(target_os = "windows"))]
    {
        shell_escape::escape(std::borrow::Cow::Borrowed(path)).into_owned()
    }
}

/// Open a terminal at the specified directory.
///
/// This is the terminal equivalent of `open_vscode` - it opens a new terminal
/// window at the specified directory without running any command.
///
/// Returns Ok(true) if terminal was opened, Ok(false) if terminal is not available,
/// or Err if terminal failed to open.
pub fn open_terminal(workspace_path: &Path) -> Result<bool, WorkspaceError> {
    open_platform_terminal(workspace_path, "clear")
}

/// Open a terminal at the specified directory with a command to run.
#[cfg(target_os = "macos")]
pub fn open_platform_terminal(working_dir: &Path, command: &str) -> Result<bool, WorkspaceError> {
    // Use osascript to open Terminal.app
    // 1. Shell-escape the path (handles spaces, $, `, ', etc.)
    let escaped_path = escape_path_for_shell(&working_dir.to_string_lossy());
    let shell_cmd = format!("cd {escaped_path} && {command}");
    // 2. Escape for AppleScript double-quoted string context (\ and ")
    let escaped_cmd = shell_cmd.replace('\\', "\\\\").replace('"', "\\\"");

    let script = format!(
        r#"tell application "Terminal"
    activate
    do script "{escaped_cmd}"
end tell"#
    );

    let result = Command::new("osascript").arg("-e").arg(&script).spawn();

    match result {
        Ok(_) => Ok(true),
        Err(e) => Err(WorkspaceError::TerminalError(e.to_string())),
    }
}

#[cfg(target_os = "linux")]
pub fn open_platform_terminal(working_dir: &Path, command: &str) -> Result<bool, WorkspaceError> {
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
        let escaped_path = escape_path_for_shell(&working_dir.to_string_lossy());
        let result = Command::new("xterm")
            .arg("-e")
            .arg(format!("cd {escaped_path} && {command}; exec bash"))
            .spawn();

        if result.is_ok() {
            return Ok(true);
        }
    }

    Err(WorkspaceError::TerminalNotFound)
}

#[cfg(target_os = "windows")]
pub fn open_platform_terminal(working_dir: &Path, command: &str) -> Result<bool, WorkspaceError> {
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
    let escaped_path = escape_path_for_shell(&working_dir.to_string_lossy());
    let cmd_script = format!("cd /d {} && {}", escaped_path, command);

    let result = Command::new("cmd").arg("/k").arg(&cmd_script).spawn();

    match result {
        Ok(_) => Ok(true),
        Err(e) => Err(WorkspaceError::TerminalError(e.to_string())),
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub fn open_platform_terminal(_working_dir: &Path, _command: &str) -> Result<bool, WorkspaceError> {
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

    #[test]
    fn test_escape_path_for_shell_simple() {
        let result = escape_path_for_shell("/simple/path");
        // Simple paths without special chars may be returned unquoted or quoted
        assert!(result.contains("/simple/path"));
    }

    #[test]
    fn test_escape_path_for_shell_spaces() {
        let result = escape_path_for_shell("/path/with spaces/dir");
        // Must be quoted/escaped to protect the space
        assert_ne!(result, "/path/with spaces/dir");
        assert!(result.contains("with spaces"));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_escape_path_for_shell_dollar_sign() {
        let result = escape_path_for_shell("/home/user/my$project");
        // Dollar sign must be quoted to prevent shell variable expansion
        assert_ne!(result, "/home/user/my$project");
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_escape_path_for_shell_single_quote() {
        let result = escape_path_for_shell("/home/user/it's a dir");
        // Single quote must be properly handled
        assert!(result.contains("it"));
        assert!(result.contains("s a dir"));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_escape_path_for_shell_backtick() {
        let result = escape_path_for_shell("/home/user/dir`cmd`");
        // Backticks must be quoted to prevent command substitution
        assert_ne!(result, "/home/user/dir`cmd`");
    }
}
