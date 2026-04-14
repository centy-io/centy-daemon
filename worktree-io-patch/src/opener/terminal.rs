use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};

fn spawn_prog(cmd: &str, args: &[&str]) -> Result<bool> {
    Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(true)
}

/// Write a bootstrap script (hook + `exec "${SHELL:-sh}"`) to a temp file and
/// spawn the terminal running it. Returns `true` if the command was recognised
/// as a terminal emulator, `false` otherwise (IDE / unknown command).
pub(super) fn try_terminal_with_init(
    path: &Path,
    command: &str,
    init_script: &str,
) -> Result<bool> {
    let path_str = path
        .to_str()
        .context("Workspace path contains non-UTF-8 characters")?;
    let cmd_lower = command.to_ascii_lowercase();
    #[cfg(windows)]
    if cmd_lower.starts_with("wt") {
        return super::wt::spawn(path_str, init_script);
    }
    let path_escaped = path_str.replace('\'', "'\\''");
    // Single quotes around {path_escaped} are shell quoting, not Rust string delimiters.
    #[allow(clippy::literal_string_with_formatting_args)]
    let bootstrap = format!(
        "#!/bin/sh\ncd '{path_escaped}'\ntrap 'exec \"${{SHELL:-sh}}\"' INT\n{init_script}\nexec \"${{SHELL:-sh}}\"\n"
    );

    let tmp_path =
        std::env::temp_dir().join(format!("worktree-hook-open-{}.sh", uuid::Uuid::new_v4()));
    std::fs::write(&tmp_path, bootstrap.as_bytes())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755))?;
    }

    let tmp_str = tmp_path
        .to_str()
        .context("Temp path contains non-UTF-8 characters")?;
    if cmd_lower.contains("iterm") {
        let script = format!(
            r#"tell application "iTerm2" to create window with default profile command "sh {tmp_str}""#
        );
        Command::new("osascript")
            .args(["-e", &script])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(true)
    } else if cmd_lower.contains("open -a terminal") {
        spawn_prog("open", &["-a", "Terminal", tmp_str])
    } else if cmd_lower.starts_with("alacritty") {
        spawn_prog(
            "alacritty",
            &["--working-directory", path_str, "-e", "sh", tmp_str],
        )
    } else if cmd_lower.starts_with("kitty") {
        spawn_prog("kitty", &["--directory", path_str, "sh", tmp_str])
    } else if cmd_lower.starts_with("wezterm") {
        spawn_prog(
            "wezterm",
            &["start", "--cwd", path_str, "--", "sh", tmp_str],
        )
    } else if cmd_lower.contains("ghostty") {
        spawn_prog("ghostty", &["-e", "sh", tmp_str])
    } else if cmd_lower.starts_with("tmux") {
        let parent_name = path.parent().and_then(|p| p.file_name());
        let sn = parent_name.and_then(|n| n.to_str()).unwrap_or("session");
        let file_name = path.file_name();
        let wn = file_name.and_then(|n| n.to_str()).unwrap_or("worktree");
        let exists = Command::new("tmux")
            .args(["has-session", "-t", sn])
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success());
        let sub = if exists { "new-window" } else { "new-session" };
        let flag = if exists { "-t" } else { "-s" };
        let tmux_args = [sub, "-d", flag, sn, "-n", wn, "-c", path_str, "sh", tmp_str];
        spawn_prog("tmux", &tmux_args)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
#[path = "terminal_tests.rs"]
mod tests;
