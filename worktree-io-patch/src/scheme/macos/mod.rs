mod install;

pub use install::install;

use anyhow::{Context, Result};

use super::SchemeStatus;

pub(super) const LSREGISTER: &str =
    "/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/\
     LaunchServices.framework/Versions/A/Support/lsregister";

pub(super) fn app_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~"))
        .join("Applications")
        .join("WorktreeRunner.app")
}

fn launch_agent_plist_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| {
        h.join("Library")
            .join("LaunchAgents")
            .join("io.worktree.runner.plist")
    })
}

pub fn uninstall() -> Result<()> {
    let app = app_dir();
    if app.exists() {
        // LLVM_COV_EXCL_START
        let _ = std::process::Command::new(LSREGISTER)
            .args(["-u"])
            .arg(&app)
            .status();
        std::fs::remove_dir_all(&app)
            .with_context(|| format!("Failed to remove {}", app.display()))?;
        println!("Removed {}", app.display());
        // LLVM_COV_EXCL_STOP
    } else {
        println!("Not installed — nothing to remove."); // LLVM_COV_EXCL_LINE
    }
    if let Some(plist) = launch_agent_plist_path() {
        // LLVM_COV_EXCL_START
        if plist.exists() {
            std::fs::remove_file(&plist).with_context(|| {
                format!("Failed to remove LaunchAgent plist at {}", plist.display())
            })?;
            println!("Removed LaunchAgent {}", plist.display());
        }
        // LLVM_COV_EXCL_STOP
    }
    Ok(())
}

/// Check whether the URL scheme handler app bundle is installed.
///
/// Always succeeds on macOS; the `Result` return type matches the platform
/// abstraction in `scheme/mod.rs`.
#[allow(clippy::unnecessary_wraps)]
pub fn status() -> Result<SchemeStatus> {
    let app = app_dir();
    if app.join("Contents").join("Info.plist").exists() {
        Ok(SchemeStatus::Installed {
            path: app.display().to_string(),
        }) // LLVM_COV_EXCL_LINE
    } else {
        Ok(SchemeStatus::NotInstalled) // LLVM_COV_EXCL_LINE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_app_dir_ends_with_app() {
        let d = app_dir();
        assert!(d.to_string_lossy().ends_with("WorktreeRunner.app"));
    }
    #[test]
    fn test_status_returns_ok() {
        assert!(status().is_ok());
    }
}
