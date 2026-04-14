use anyhow::{bail, Context, Result};

pub(super) fn applescript_quoted(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

pub(super) fn plist_buddy(pb: &str, cmd: &str, plist: &std::path::Path) -> Result<()> {
    let status = std::process::Command::new(pb)
        .args(["-c", cmd])
        .arg(plist)
        .status()
        .with_context(|| format!("Failed to run PlistBuddy: {cmd}"))?;
    if !status.success() {
        bail!("PlistBuddy failed: {cmd}");
    }
    Ok(())
}
