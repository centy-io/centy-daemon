pub(super) use worktree_io::opener::resolve_editor_command;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_known_editors() {
        assert_eq!(resolve_editor_command("cursor"), "cursor .");
        assert_eq!(resolve_editor_command("CODE"), "code .");
        assert_eq!(resolve_editor_command("vscode"), "code .");
        assert_eq!(resolve_editor_command("nvim"), "nvim .");
        assert_eq!(resolve_editor_command("iterm2"), "open -a iTerm .");
        assert_eq!(resolve_editor_command("warp"), "open -a Warp .");
        assert_eq!(resolve_editor_command("ghostty"), "open -a Ghostty .");
        assert_eq!(
            resolve_editor_command("alacritty"),
            "alacritty --working-directory ."
        );
        assert_eq!(resolve_editor_command("kitty"), "kitty --directory .");
        assert_eq!(resolve_editor_command("wezterm"), "wezterm start --cwd .");
        #[cfg(target_os = "windows")]
        assert_eq!(resolve_editor_command("wt"), "wt -d .");
        #[cfg(target_os = "windows")]
        assert_eq!(resolve_editor_command("windowsterminal"), "wt -d .");
    }
    #[test]
    fn test_terminal_macos() {
        let cmd = resolve_editor_command("terminal");
        assert!(!cmd.is_empty());
    }
    #[test]
    fn test_unknown_passthrough() {
        assert_eq!(resolve_editor_command("hx ."), "hx .");
    }
}
