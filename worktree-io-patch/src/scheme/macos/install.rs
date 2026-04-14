use anyhow::{bail, Context, Result};
use std::process::Command;
#[path = "helpers.rs"]
mod helpers;
use helpers::{applescript_quoted, plist_buddy};
#[path = "launch_agent.rs"]
mod launch_agent;
use launch_agent::install_launch_agent;
/// Embedded custom icon for the `WorktreeRunner.app` bundle.
static APPLET_ICNS: &[u8] = include_bytes!("../../../assets/applet.icns");

pub fn install() -> Result<()> {
    let exe = std::env::current_exe().context("Failed to get current executable path")?;
    let app = super::app_dir();
    if app.exists() {
        std::fs::remove_dir_all(&app)
            .with_context(|| format!("Failed to remove existing app at {}", app.display()))?;
    }
    let script_src = std::env::temp_dir().join("worktree-runner.applescript");
    let applescript = format!(
        "on open location this_URL\n\
         \tdo shell script {exe_q} & \" open \" & quoted form of this_URL\n\
         end open location\n",
        exe_q = applescript_quoted(&exe.display().to_string()),
    );
    std::fs::write(&script_src, &applescript).context("Failed to write AppleScript source")?;
    let status = Command::new("osacompile")
        .args(["-o"])
        .arg(&app)
        .arg(&script_src)
        .status()
        .context("Failed to run osacompile")?;
    let _ = std::fs::remove_file(&script_src);
    if !status.success() {
        bail!("osacompile failed");
    }
    // Replace the default applet icon with the custom WorktreeRunner icon.
    let icon_dest = app.join("Contents").join("Resources").join("applet.icns");
    std::fs::write(&icon_dest, APPLET_ICNS)
        .with_context(|| format!("Failed to write icon to {}", icon_dest.display()))?;
    let plist = app.join("Contents").join("Info.plist");
    let pb = "/usr/libexec/PlistBuddy";
    plist_buddy(
        pb,
        "Add :CFBundleIdentifier string io.worktree.runner",
        &plist,
    )?;
    plist_buddy(pb, "Set :CFBundleName WorktreeRunner", &plist)?;
    let _ = Command::new(pb)
        .args(["-c", "Add :LSUIElement bool true"])
        .arg(&plist)
        .status();
    plist_buddy(pb, "Set :LSUIElement true", &plist)?;
    let _ = Command::new(pb)
        .args(["-c", "Add :CFBundleURLTypes array"])
        .arg(&plist)
        .status();
    plist_buddy(pb, "Add :CFBundleURLTypes:0 dict", &plist)?;
    plist_buddy(
        pb,
        "Add :CFBundleURLTypes:0:CFBundleURLName string Worktree URL",
        &plist,
    )?;
    plist_buddy(
        pb,
        "Add :CFBundleURLTypes:0:CFBundleURLSchemes array",
        &plist,
    )?;
    plist_buddy(
        pb,
        "Add :CFBundleURLTypes:0:CFBundleURLSchemes:0 string worktree",
        &plist,
    )?;
    // Ad-hoc sign the bundle so Gatekeeper does not quarantine or evict it.
    let status = Command::new("codesign")
        .args(["--sign", "-", "--force", "--deep"])
        .arg(&app)
        .status()
        .context("Failed to run codesign")?;
    if !status.success() {
        bail!("codesign failed");
    }
    let lsregister = super::LSREGISTER;
    let status = Command::new(lsregister)
        .arg("-f")
        .arg(&app)
        .status()
        .context("Failed to run lsregister")?;
    if !status.success() {
        bail!("lsregister failed");
    }
    install_launch_agent(&app)?;
    println!("Installed WorktreeRunner.app at {}", app.display());
    println!("The worktree:// URL scheme is now registered.");
    Ok(())
}

#[cfg(test)]
#[path = "install_tests.rs"]
mod tests;
