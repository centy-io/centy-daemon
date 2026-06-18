use worktree_io::opener::available_entries::available_entries;

pub(super) fn detect_all_editors() -> Vec<(&'static str, &'static str)> {
    available_entries()
        .into_iter()
        .map(|e| (e.display, e.command))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_detect_all_editors_returns_vec() {
        let editors = detect_all_editors();
        assert!(editors.iter().any(|&(name, _)| name == "Terminal"));
    }
}
