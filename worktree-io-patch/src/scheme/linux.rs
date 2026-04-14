use anyhow::{Context, Result};
use std::process::Command;

use super::SchemeStatus;

static ICON_PNG: &[u8] = include_bytes!("../../assets/logo.png");

fn desktop_file() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.local/share"))
        .join("applications")
        .join("worktree-runner.desktop")
}

fn icon_file() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.local/share"))
        .join("icons/hicolor/256x256/apps/worktree-runner.png")
}

pub fn install() -> Result<()> {
    let exe = std::env::current_exe().context("Failed to get current executable path")?;
    let path = desktop_file();
    let icon = icon_file();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    if let Some(parent) = icon.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    std::fs::write(&icon, ICON_PNG)
        .with_context(|| format!("Failed to write icon to {}", icon.display()))?;
    let content = format!(
        "[Desktop Entry]\nName=Worktree Runner\nExec={exe} open %u\n\
         Type=Application\nNoDisplay=true\nIcon=worktree-runner\n\
         MimeType=x-scheme-handler/worktree;\n",
        exe = exe.display()
    );
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write desktop file to {}", path.display()))?;
    Command::new("xdg-mime")
        .args([
            "default",
            "worktree-runner.desktop",
            "x-scheme-handler/worktree",
        ])
        .status()
        .context("Failed to run xdg-mime")?;
    println!("Installed desktop entry at {}", path.display());
    Ok(())
}

pub fn uninstall() -> Result<()> {
    let path = desktop_file();
    if path.exists() {
        std::fs::remove_file(&path)
            .with_context(|| format!("Failed to remove {}", path.display()))?;
        println!("Removed {}", path.display());
    } else {
        println!("Not installed — nothing to remove.");
    }
    let icon = icon_file();
    if icon.exists() {
        std::fs::remove_file(&icon)
            .with_context(|| format!("Failed to remove {}", icon.display()))?;
        println!("Removed {}", icon.display());
    }
    Ok(())
}

pub fn status() -> Result<SchemeStatus> {
    let path = desktop_file();
    if path.exists() {
        Ok(SchemeStatus::Installed {
            path: path.display().to_string(),
        })
    } else {
        Ok(SchemeStatus::NotInstalled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_desktop_file_name() {
        assert!(desktop_file()
            .to_string_lossy()
            .ends_with("worktree-runner.desktop"));
    }
    #[test]
    fn test_icon_file_name() {
        assert!(icon_file()
            .to_string_lossy()
            .ends_with("worktree-runner.png"));
    }
}
