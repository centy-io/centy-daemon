/// How to check whether an editor/terminal is available on the current system.
pub enum DetectMethod {
    /// Binary name to look up in `PATH`.
    Path(&'static str),
    /// macOS `.app` bundle name (checked in `/Applications` and `~/Applications`).
    #[cfg(target_os = "macos")]
    MacosApp(&'static str),
    /// Unconditionally considered present (e.g. Terminal.app on macOS).
    Always,
}

/// A single supported editor or terminal option.
pub struct EditorEntry {
    /// Short aliases accepted by `resolve_editor_command` (case-insensitive).
    pub aliases: &'static [&'static str],
    /// Human-readable name shown in the setup prompt.
    pub display: &'static str,
    /// Shell command passed to the opener (`.` is replaced with the actual path).
    pub command: &'static str,
    /// How to check whether this option is available on the current system.
    pub detect: DetectMethod,
}

// (aliases, display, command, binary-to-detect-in-PATH)
const BASE: &[(&[&str], &str, &str, &str)] = &[
    (&["cursor"], "Cursor", "cursor .", "cursor"),
    (&["vscode", "code"], "VS Code", "code .", "code"),
    (&["zed"], "Zed", "zed .", "zed"),
    (&["subl"], "Sublime Text", "subl .", "subl"),
    (&["nvim"], "Neovim", "nvim .", "nvim"),
    (&["vim"], "Vim", "vim .", "vim"),
    (&["tmux"], "tmux", "tmux", "tmux"),
    (
        &["alacritty"],
        "Alacritty",
        "alacritty --working-directory .",
        "alacritty",
    ),
    (&["kitty"], "Kitty", "kitty --directory .", "kitty"),
    (&["wezterm"], "WezTerm", "wezterm start --cwd .", "wezterm"),
];

/// All supported editor/terminal entries for the current platform.
#[must_use]
#[allow(unused_mut)]
pub fn all_entries() -> Vec<EditorEntry> {
    let mut v: Vec<EditorEntry> = BASE
        .iter()
        .map(|&(aliases, display, command, binary)| EditorEntry {
            aliases,
            display,
            command,
            detect: DetectMethod::Path(binary),
        })
        .collect();
    #[cfg(target_os = "macos")]
    {
        use DetectMethod::{Always, MacosApp};
        v.push(EditorEntry {
            aliases: &["terminal"],
            display: "Terminal",
            command: "open -a Terminal .",
            detect: Always,
        });
        v.push(EditorEntry {
            aliases: &["iterm", "iterm2"],
            display: "iTerm2",
            command: "open -a iTerm .",
            detect: MacosApp("iTerm"),
        });
        v.push(EditorEntry {
            aliases: &["warp"],
            display: "Warp",
            command: "open -a Warp .",
            detect: MacosApp("Warp"),
        });
        v.push(EditorEntry {
            aliases: &["ghostty"],
            display: "Ghostty",
            command: "open -a Ghostty .",
            detect: MacosApp("Ghostty"),
        });
    }
    #[cfg(target_os = "windows")]
    v.push(EditorEntry {
        aliases: &["wt", "windowsterminal", "terminal"],
        display: "Windows Terminal",
        command: "wt -d .",
        detect: DetectMethod::Path("wt"),
    });
    v
}
