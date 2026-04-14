use anyhow::{bail, Context, Result};
use std::process::Command;

use super::SchemeStatus;

pub fn install() -> Result<()> {
    let exe = std::env::current_exe()
        .context("Failed to get current executable path")?
        .display()
        .to_string();

    let run = |args: &[&str]| -> Result<()> {
        let status = Command::new("reg")
            .args(args)
            .status()
            .context("Failed to run `reg`")?;
        if !status.success() {
            bail!("reg command failed");
        }
        Ok(())
    };

    run(&[
        "add",
        r"HKCU\Software\Classes\worktree",
        "/d",
        "URL:Worktree Protocol",
        "/f",
    ])?;
    run(&[
        "add",
        r"HKCU\Software\Classes\worktree",
        "/v",
        "URL Protocol",
        "/d",
        "",
        "/f",
    ])?;
    run(&[
        "add",
        r"HKCU\Software\Classes\worktree\shell\open\command",
        "/d",
        &format!(r#""{exe}" open "%1""#),
        "/f",
    ])?;

    println!("Registered worktree:// URL scheme in Windows registry.");
    Ok(())
}

pub fn uninstall() -> Result<()> {
    let status = Command::new("reg")
        .args(["delete", r"HKCU\Software\Classes\worktree", "/f"])
        .status()
        .context("Failed to run `reg delete`")?;

    if !status.success() {
        bail!("reg delete failed");
    }
    println!("Unregistered worktree:// URL scheme.");
    Ok(())
}

pub fn status() -> Result<SchemeStatus> {
    let output = Command::new("reg")
        .args(["query", r"HKCU\Software\Classes\worktree"])
        .output()
        .context("Failed to query registry")?;

    if output.status.success() {
        Ok(SchemeStatus::Installed {
            path: r"HKCU\Software\Classes\worktree".to_string(),
        })
    } else {
        Ok(SchemeStatus::NotInstalled)
    }
}
