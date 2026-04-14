use super::entries::{all_entries, EditorEntry};
use super::is_available::is_available;

/// Return only the editors/terminals available on the current system.
#[must_use]
pub fn available_entries() -> Vec<EditorEntry> {
    all_entries()
        .into_iter()
        .filter(|e| is_available(&e.detect))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_entries_returns_vec() {
        let editors = available_entries();
        assert!(editors.iter().any(|e| e.display == "Terminal"));
    }
}
