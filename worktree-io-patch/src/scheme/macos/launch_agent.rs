use anyhow::{Context, Result};
use std::process::Command;

pub(super) fn install_launch_agent(app: &std::path::Path) -> Result<()> {
    let agents_dir = dirs::home_dir()
        .context("Failed to get home directory")?
        .join("Library")
        .join("LaunchAgents");
    std::fs::create_dir_all(&agents_dir).context("Failed to create LaunchAgents directory")?;
    let plist_path = agents_dir.join("io.worktree.runner.plist");
    std::fs::write(&plist_path, launch_agent_plist_content(app)).with_context(|| {
        format!(
            "Failed to write LaunchAgent plist at {}",
            plist_path.display()
        )
    })?;
    let uid = Command::new("id")
        .arg("-u")
        .output()
        .context("Failed to run `id -u`")?;
    let uid = String::from_utf8_lossy(&uid.stdout).trim().to_string();
    let domain = format!("gui/{uid}");
    // Unload any existing instance before bootstrapping (ignore errors).
    let _ = Command::new("launchctl")
        .args(["bootout", &domain])
        .arg(&plist_path)
        .status();
    // Load the agent immediately so the current session is covered.
    let _ = Command::new("launchctl")
        .args(["bootstrap", &domain])
        .arg(&plist_path)
        .status();
    Ok(())
}

pub(super) fn launch_agent_plist_content(app: &std::path::Path) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n\
         <dict>\n\
         \t<key>Label</key>\n\
         \t<string>io.worktree.runner</string>\n\
         \t<key>ProgramArguments</key>\n\
         \t<array>\n\
         \t\t<string>{lsregister}</string>\n\
         \t\t<string>-f</string>\n\
         \t\t<string>{app}</string>\n\
         \t</array>\n\
         \t<key>RunAtLoad</key>\n\
         \t<true/>\n\
         \t<key>StartInterval</key>\n\
         \t<integer>3600</integer>\n\
         </dict>\n\
         </plist>\n",
        lsregister = super::super::LSREGISTER,
        app = app.display(),
    )
}
