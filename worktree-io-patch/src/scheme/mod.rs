use anyhow::Result;

mod dispatch;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

/// Whether the `worktree://` URL scheme handler is registered on this system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemeStatus {
    /// The handler is installed at the given path.
    Installed {
        /// File-system path to the installed application bundle or binary.
        path: String,
    },
    /// The handler is not installed.
    NotInstalled,
}

impl std::fmt::Display for SchemeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Installed { path } => write!(f, "Installed at {path}"),
            Self::NotInstalled => write!(f, "Not installed"),
        }
    }
}

/// Register the `worktree://` URL scheme handler on the current platform.
///
/// # Errors
///
/// Returns an error if the platform is unsupported or registration fails.
pub fn install() -> Result<()> {
    dispatch::platform_install()
}

/// Remove the `worktree://` URL scheme handler from the current platform.
///
/// # Errors
///
/// Returns an error if the platform is unsupported or removal fails.
pub fn uninstall() -> Result<()> {
    dispatch::platform_uninstall()
}

/// Query whether the `worktree://` URL scheme handler is currently installed.
///
/// # Errors
///
/// Returns an error if the platform is unsupported or the query fails.
pub fn status() -> Result<SchemeStatus> {
    dispatch::platform_status()
}

#[cfg(test)]
#[path = "scheme_tests.rs"]
mod tests;
