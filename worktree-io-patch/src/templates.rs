/// Scaffold written to `.worktree.toml` when the file is absent.
pub const WORKTREE_TOML: &str = include_str!("../assets/worktree.toml");

/// Default commented config written to `~/.config/worktree/config.toml`.
pub const CONFIG_TOML: &str = include_str!("../assets/config.toml");

/// Reference template showing the format of the workspaces registry file.
pub const WORKSPACES_TOML: &str = include_str!("../assets/workspaces.toml");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    #[test]
    fn config_toml_in_sync_with_default() {
        assert_eq!(CONFIG_TOML, Config::default().to_toml_with_comments());
    }

    #[test]
    fn worktree_toml_starts_with_comment() {
        assert!(WORKTREE_TOML.starts_with('#'));
    }

    #[test]
    fn workspaces_toml_starts_with_comment() {
        assert!(WORKSPACES_TOML.starts_with('#'));
    }
}
