//! Tests for link/storage/io.rs covering all branches.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::io::{read_links, write_links};
use super::links_file::{LinksFile, LINKS_FILENAME};
use crate::link::{Link, TargetType};
use tokio::fs;

// ─── read_links ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_read_links_no_file_returns_empty() {
    let temp = tempfile::tempdir().unwrap();
    let issues_dir = temp.path().join("issues");
    fs::create_dir_all(&issues_dir).await.unwrap();
    let entity_path = issues_dir.join("my-entity");

    let result = read_links(&entity_path).await.unwrap();
    assert!(result.links.is_empty());
}

#[tokio::test]
async fn test_read_links_old_style_path() {
    // Old-style: entity_path/links.json
    let temp = tempfile::tempdir().unwrap();
    let entity_path = temp.path().join("issues").join("uuid-1");
    fs::create_dir_all(&entity_path).await.unwrap();

    let mut links = LinksFile::new();
    links.add_link(Link::new(
        "uuid-2".to_string(),
        TargetType::issue(),
        "blocks".to_string(),
    ));
    let json = serde_json::to_string_pretty(&links).unwrap();
    fs::write(entity_path.join(LINKS_FILENAME), json)
        .await
        .unwrap();

    let result = read_links(&entity_path).await.unwrap();
    assert_eq!(result.links.len(), 1);
    assert_eq!(result.links[0].kind, "blocks");
    assert_eq!(result.links[0].target_id, "uuid-2");
}

#[tokio::test]
async fn test_read_links_old_style_invalid_json_error() {
    let temp = tempfile::tempdir().unwrap();
    let entity_path = temp.path().join("issues").join("uuid-bad");
    fs::create_dir_all(&entity_path).await.unwrap();

    fs::write(entity_path.join(LINKS_FILENAME), b"{ invalid json }")
        .await
        .unwrap();

    let result = read_links(&entity_path).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
}

#[tokio::test]
async fn test_read_links_new_style_path() {
    // New-style: parent/links/{entity_id}/links.json
    let temp = tempfile::tempdir().unwrap();
    let entity_path = temp.path().join("issues").join("uuid-new");
    // Don't create entity_path directory — so old-style won't match

    let links_dir = temp.path().join("issues").join("links").join("uuid-new");
    fs::create_dir_all(&links_dir).await.unwrap();

    let mut links = LinksFile::new();
    links.add_link(Link::new(
        "uuid-3".to_string(),
        TargetType::issue(),
        "relates-to".to_string(),
    ));
    let json = serde_json::to_string_pretty(&links).unwrap();
    fs::write(links_dir.join(LINKS_FILENAME), json)
        .await
        .unwrap();

    let result = read_links(&entity_path).await.unwrap();
    assert_eq!(result.links.len(), 1);
    assert_eq!(result.links[0].kind, "relates-to");
}

#[tokio::test]
async fn test_read_links_new_style_invalid_json_error() {
    let temp = tempfile::tempdir().unwrap();
    let entity_path = temp.path().join("issues").join("uuid-bad-new");

    let links_dir = temp
        .path()
        .join("issues")
        .join("links")
        .join("uuid-bad-new");
    fs::create_dir_all(&links_dir).await.unwrap();
    fs::write(links_dir.join(LINKS_FILENAME), b"not json at all")
        .await
        .unwrap();

    let result = read_links(&entity_path).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);
}

// ─── write_links ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_write_links_with_links_creates_file() {
    let temp = tempfile::tempdir().unwrap();
    let entity_path = temp.path().join("issues").join("uuid-1");

    let mut links = LinksFile::new();
    links.add_link(Link::new(
        "uuid-2".to_string(),
        TargetType::issue(),
        "blocks".to_string(),
    ));

    write_links(&entity_path, &links).await.unwrap();

    let links_file = temp
        .path()
        .join("issues")
        .join("links")
        .join("uuid-1")
        .join(LINKS_FILENAME);
    assert!(links_file.exists());

    let content = fs::read_to_string(&links_file).await.unwrap();
    let parsed: LinksFile = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.links.len(), 1);
}

#[tokio::test]
async fn test_write_links_empty_removes_new_file() {
    let temp = tempfile::tempdir().unwrap();
    let entity_path = temp.path().join("issues").join("uuid-1");

    // First write with a link
    let mut links = LinksFile::new();
    links.add_link(Link::new(
        "uuid-2".to_string(),
        TargetType::issue(),
        "blocks".to_string(),
    ));
    write_links(&entity_path, &links).await.unwrap();

    let links_file = temp
        .path()
        .join("issues")
        .join("links")
        .join("uuid-1")
        .join(LINKS_FILENAME);
    assert!(links_file.exists());

    // Then write empty
    let empty = LinksFile::new();
    write_links(&entity_path, &empty).await.unwrap();
    assert!(!links_file.exists());
}

#[tokio::test]
async fn test_write_links_empty_with_no_file_is_ok() {
    let temp = tempfile::tempdir().unwrap();
    let entity_path = temp.path().join("issues").join("uuid-no-file");

    // Write empty when there's no existing file — should succeed silently
    let empty = LinksFile::new();
    let result = write_links(&entity_path, &empty).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_write_links_empty_removes_old_style_file() {
    let temp = tempfile::tempdir().unwrap();
    let entity_path = temp.path().join("issues").join("uuid-legacy");
    fs::create_dir_all(&entity_path).await.unwrap();

    // Create old-style links.json
    let old_links = entity_path.join(LINKS_FILENAME);
    let mut links = LinksFile::new();
    links.add_link(Link::new(
        "uuid-x".to_string(),
        TargetType::issue(),
        "blocks".to_string(),
    ));
    fs::write(&old_links, serde_json::to_string_pretty(&links).unwrap())
        .await
        .unwrap();
    assert!(old_links.exists());

    // Now write empty links — should remove old file
    write_links(&entity_path, &LinksFile::new()).await.unwrap();
    assert!(!old_links.exists());
}

#[tokio::test]
async fn test_write_links_migrates_old_to_new_and_removes_old() {
    let temp = tempfile::tempdir().unwrap();
    let entity_path = temp.path().join("issues").join("uuid-migrate");
    fs::create_dir_all(&entity_path).await.unwrap();

    // Create old-style links.json
    let old_links = entity_path.join(LINKS_FILENAME);
    let mut existing = LinksFile::new();
    existing.add_link(Link::new(
        "uuid-old".to_string(),
        TargetType::issue(),
        "relates-to".to_string(),
    ));
    fs::write(&old_links, serde_json::to_string_pretty(&existing).unwrap())
        .await
        .unwrap();
    assert!(old_links.exists());

    // Write new links (non-empty) — should write to new path and remove old
    let mut new_links = LinksFile::new();
    new_links.add_link(Link::new(
        "uuid-new".to_string(),
        TargetType::issue(),
        "parent-of".to_string(),
    ));
    write_links(&entity_path, &new_links).await.unwrap();

    // Old file should be gone
    assert!(!old_links.exists());

    // New file should exist
    let new_links_path = temp
        .path()
        .join("issues")
        .join("links")
        .join("uuid-migrate")
        .join(LINKS_FILENAME);
    assert!(new_links_path.exists());
}

#[tokio::test]
async fn test_write_links_roundtrip_read_write_read() {
    let temp = tempfile::tempdir().unwrap();
    let entity_path = temp.path().join("issues").join("uuid-rt");

    let mut links = LinksFile::new();
    links.add_link(Link::new(
        "a".to_string(),
        TargetType::issue(),
        "blocks".to_string(),
    ));
    links.add_link(Link::new(
        "b".to_string(),
        TargetType::new("doc"),
        "relates-to".to_string(),
    ));

    write_links(&entity_path, &links).await.unwrap();

    let read_back = read_links(&entity_path).await.unwrap();
    assert_eq!(read_back.links.len(), 2);
}

#[tokio::test]
async fn test_write_links_empty_removes_empty_links_dir() {
    let temp = tempfile::tempdir().unwrap();
    let entity_path = temp.path().join("issues").join("uuid-cleanup");

    // Write links first
    let mut links = LinksFile::new();
    links.add_link(Link::new(
        "uuid-x".to_string(),
        TargetType::issue(),
        "blocks".to_string(),
    ));
    write_links(&entity_path, &links).await.unwrap();

    let links_dir = temp
        .path()
        .join("issues")
        .join("links")
        .join("uuid-cleanup");
    assert!(links_dir.exists());

    // Now write empty — dir should be cleaned up
    write_links(&entity_path, &LinksFile::new()).await.unwrap();
    assert!(!links_dir.exists(), "Empty links dir should be removed");
}
