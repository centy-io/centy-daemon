use super::*;
use crate::link::TargetType;

#[test]
fn test_links_file_new() {
    let file = LinksFile::new();
    assert!(file.links.is_empty());
}

#[test]
fn test_links_file_add_link() {
    let mut file = LinksFile::new();
    file.add_link(Link::new(
        "uuid-1".to_string(),
        TargetType::Issue,
        "blocks".to_string(),
    ));
    assert_eq!(file.links.len(), 1);
}

#[test]
fn test_links_file_remove_link() {
    let mut file = LinksFile::new();
    file.add_link(Link::new(
        "uuid-1".to_string(),
        TargetType::Issue,
        "blocks".to_string(),
    ));
    file.add_link(Link::new(
        "uuid-1".to_string(),
        TargetType::Issue,
        "parent-of".to_string(),
    ));

    // Remove specific link type
    assert!(file.remove_link("uuid-1", Some("blocks")));
    assert_eq!(file.links.len(), 1);
    assert_eq!(file.links[0].link_type, "parent-of");
}

#[test]
fn test_links_file_remove_all_links_to_target() {
    let mut file = LinksFile::new();
    file.add_link(Link::new(
        "uuid-1".to_string(),
        TargetType::Issue,
        "blocks".to_string(),
    ));
    file.add_link(Link::new(
        "uuid-1".to_string(),
        TargetType::Issue,
        "parent-of".to_string(),
    ));
    file.add_link(Link::new(
        "uuid-2".to_string(),
        TargetType::Doc,
        "relates-to".to_string(),
    ));

    // Remove all links to uuid-1
    assert!(file.remove_link("uuid-1", None));
    assert_eq!(file.links.len(), 1);
    assert_eq!(file.links[0].target_id, "uuid-2");
}

#[test]
fn test_links_file_has_link() {
    let mut file = LinksFile::new();
    file.add_link(Link::new(
        "uuid-1".to_string(),
        TargetType::Issue,
        "blocks".to_string(),
    ));

    assert!(file.has_link("uuid-1", "blocks"));
    assert!(!file.has_link("uuid-1", "parent-of"));
    assert!(!file.has_link("uuid-2", "blocks"));
}

#[test]
fn test_links_file_serialization() {
    let mut file = LinksFile::new();
    file.add_link(Link::new(
        "uuid-1".to_string(),
        TargetType::Issue,
        "blocks".to_string(),
    ));

    let json = serde_json::to_string_pretty(&file).unwrap();
    assert!(json.contains("\"links\""));
    assert!(json.contains("\"targetId\": \"uuid-1\""));

    let parsed: LinksFile = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.links.len(), 1);
}

