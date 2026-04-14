use anyhow::{Context, Result};
use std::process::{Command, Stdio};

/// Write a `.bat` bootstrap and launch Windows Terminal (`wt`) running it.
pub(super) fn spawn(path_str: &str, init_script: &str) -> Result<bool> {
    let tmp = std::env::temp_dir().join(format!("worktree-hook-open-{}.bat", uuid::Uuid::new_v4()));
    let bat = format!(
        "@echo off\r\ncd /d \"{}\"\r\n{init_script}\r\ncmd /k\r\n",
        path_str.replace('"', "\"\""),
    );
    std::fs::write(&tmp, bat.as_bytes())?;
    let s = tmp
        .to_str()
        .context("Temp path contains non-UTF-8 characters")?;
    Command::new("wt")
        .args(["cmd", "/k", s])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(true)
}
