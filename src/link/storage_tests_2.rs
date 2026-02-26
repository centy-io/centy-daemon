use super::*;
use crate::link::TargetType;

#[test]
fn test_links_file_multiple_links() {
    let mut file = LinksFile::new();
    file.add_link(Link::new(
        "uuid-1".to_string(),
        TargetType::Issue,
        "blocks".to_string(),
    ));
    file.add_link(Link::new(
        "uuid-2".to_string(),
        TargetType::Doc,
        "relates-to".to_string(),
    ));
    file.add_link(Link::new(
        "uuid-3".to_string(),
        TargetType::Issue,
        "parent-of".to_string(),
    ));

    assert_eq!(file.links.len(), 3);
    assert!(file.has_link("uuid-1", "blocks"));
    assert!(file.has_link("uuid-2", "relates-to"));
    assert!(file.has_link("uuid-3", "parent-of"));
}

#[test]
fn test_links_file_clone() {
    let mut file = LinksFile::new();
    file.add_link(Link::new(
        "uuid-1".to_string(),
        TargetType::Issue,
        "blocks".to_string(),
    ));

    let cloned = file.clone();
    assert_eq!(cloned.links.len(), 1);
    assert_eq!(cloned.links[0].target_id, "uuid-1");
}

#[test]
fn test_links_filename_constant() {
    assert_eq!(LINKS_FILENAME, "links.json");
}

#[tokio::test]
async fn test_read_links_nonexistent() {
    use tempfile::tempdir;

    let temp_dir = tempdir().expect("Should create temp dir");
    let entity_path = temp_dir.path().join("issues").join("uuid-1");

    let links = read_links(&entity_path).await.expect("Should read");
    assert!(links.links.is_empty());
}

#[tokio::test]
async fn test_write_and_read_links() {
    use tempfile::tempdir;

    let temp_dir = tempdir().expect("Should create temp dir");
    let issues_path = temp_dir.path().join("issues");
    fs::create_dir_all(&issues_path)
        .await
        .expect("Should create dirs");

    let entity_path = issues_path.join("uuid-1");

    let mut links_file = LinksFile::new();
    links_file.add_link(Link::new(
        "uuid-2".to_string(),
        TargetType::Issue,
        "blocks".to_string(),
    ));

    write_links(&entity_path, &links_file)
        .await
        .expect("Should write");

    let read_back = read_links(&entity_path).await.expect("Should read");
    assert_eq!(read_back.links.len(), 1);
    assert_eq!(read_back.links[0].target_id, "uuid-2");
    assert_eq!(read_back.links[0].link_type, "blocks");
}
