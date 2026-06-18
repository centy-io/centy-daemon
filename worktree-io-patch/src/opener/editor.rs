use super::entries::all_entries;

/// Map a short editor name (e.g. "terminal", "vscode", "cursor") to a shell command.
#[must_use]
pub fn resolve_editor_command(name: &str) -> String {
    for entry in all_entries() {
        if entry.aliases.iter().any(|&a| a.eq_ignore_ascii_case(name)) {
            return entry.command.to_string();
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    if name.eq_ignore_ascii_case("terminal") {
        return "xterm".to_string();
    }
    name.to_string()
}
