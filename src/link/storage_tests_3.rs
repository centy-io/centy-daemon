use super::*;
use crate::link::TargetType;

#[test]
fn test_links_file_default() {
    let file = LinksFile::default();
    assert!(file.links.is_empty());
}

#[test]
fn test_links_file_remove_nonexistent() {
    let mut file = LinksFile::new();
    file.add_link(Link::new(
        "uuid-1".to_string(),
        TargetType::Issue,
        "blocks".to_string(),
    ));

    assert!(!file.remove_link("uuid-999", Some("blocks")));
    assert_eq!(file.links.len(), 1);
}

#[test]
fn test_links_file_has_link_empty() {
    let file = LinksFile::new();
    assert!(!file.has_link("any-id", "any-type"));
}
