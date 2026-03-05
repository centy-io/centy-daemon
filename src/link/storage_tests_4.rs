use super::*;
use crate::link::TargetType;

#[tokio::test]
async fn test_write_empty_links_removes_file() {
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
        TargetType::issue(),
        "blocks".to_string(),
    ));
    write_links(&entity_path, &links_file)
        .await
        .expect("Should write");

    let empty_file = LinksFile::new();
    write_links(&entity_path, &empty_file)
        .await
        .expect("Should write empty");

    let read_back = read_links(&entity_path).await.expect("Should read");
    assert!(read_back.links.is_empty());
}
