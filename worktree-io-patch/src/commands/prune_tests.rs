use super::{dir_size, format_bytes};

#[test]
fn test_format_bytes_b() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(512), "512 B");
    assert_eq!(format_bytes(1_023), "1023 B");
}

#[test]
fn test_format_bytes_kb() {
    assert_eq!(format_bytes(1_024), "1.0 KB");
    assert_eq!(format_bytes(1_536), "1.5 KB");
    assert_eq!(format_bytes(10_240), "10.0 KB");
}

#[test]
fn test_format_bytes_mb() {
    assert_eq!(format_bytes(1_048_576), "1.0 MB");
    assert_eq!(format_bytes(5_242_880), "5.0 MB");
}

#[test]
fn test_format_bytes_gb() {
    assert_eq!(format_bytes(1_073_741_824), "1.0 GB");
    assert_eq!(format_bytes(2_147_483_648), "2.0 GB");
}

#[test]
fn test_dir_size_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(dir_size(dir.path()), 0);
}

#[test]
fn test_dir_size_with_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("f"), b"hello").unwrap();
    assert_eq!(dir_size(dir.path()), 5);
}

#[test]
fn test_dir_size_recursive() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("sub");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("f"), b"world").unwrap();
    assert_eq!(dir_size(dir.path()), 5);
}

#[test]
fn test_dir_size_not_a_dir() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("f");
    std::fs::write(&file, b"x").unwrap();
    assert_eq!(dir_size(&file), 0);
}
