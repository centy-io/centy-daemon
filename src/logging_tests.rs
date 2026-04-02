use super::*;

#[test]
fn test_log_config_default() {
    let config = LogConfig::default();
    assert_eq!(config.log_level, Level::INFO);
    assert!(!config.json_format);
    assert!(config.log_dir.ends_with("logs"));
    assert_eq!(config.max_log_files, DEFAULT_MAX_LOG_FILES);
}

#[test]
fn test_log_config_default_log_dir_contains_centy() {
    let config = LogConfig::default();
    let path_str = config.log_dir.to_string_lossy();
    assert!(path_str.contains(".centy"));
}

#[test]
fn test_parse_rotation_hourly() {
    let rotation = parse_rotation("hourly");
    // Rotation doesn't impl PartialEq, so use debug
    let debug = format!("{rotation:?}");
    assert!(debug.contains("Hourly") || debug.contains("hourly") || debug.contains("3600"));
}

#[test]
fn test_parse_rotation_never() {
    let rotation = parse_rotation("never");
    let debug = format!("{rotation:?}");
    assert!(debug.contains("Never") || debug.contains("never"));
}

#[test]
fn test_parse_rotation_daily() {
    let rotation = parse_rotation("daily");
    let debug = format!("{rotation:?}");
    assert!(debug.contains("Daily") || debug.contains("daily") || debug.contains("86400"));
}

#[test]
fn test_parse_rotation_case_insensitive() {
    drop(parse_rotation("HOURLY"));
    drop(parse_rotation("Never"));
    drop(parse_rotation("DAILY"));
}

#[test]
fn test_parse_rotation_unknown_defaults_to_daily() {
    let rotation = parse_rotation("weekly");
    let debug = format!("{rotation:?}");
    // Unknown values default to daily
    let daily = format!("{:?}", parse_rotation("daily"));
    assert_eq!(debug, daily);
}

#[test]
fn test_log_filename_constant() {
    assert_eq!(LOG_FILENAME, "centy-daemon.log");
}

#[test]
fn test_cleanup_old_log_files_removes_oldest() {
    use std::fs;
    use std::time::{Duration, SystemTime};

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();

    // Create 5 log files with distinct modification times.
    for i in 0..5u64 {
        let file_path = path.join(format!("centy-daemon.log.2024-01-0{}", i + 1));
        fs::write(&file_path, format!("log {i}")).unwrap();
        // Set mtime so files have a clear ordering.
        let mtime = SystemTime::UNIX_EPOCH + Duration::from_secs((i + 1) * 86400);
        fs::File::options()
            .write(true)
            .open(&file_path)
            .unwrap()
            .set_modified(mtime)
            .unwrap();
    }

    // Keep only the 3 most recent.
    cleanup_old_log_files(path, 3);

    let mut remaining: Vec<_> = fs::read_dir(path)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    remaining.sort();

    assert_eq!(remaining.len(), 3);
    // The 3 newest are 2024-01-03, 2024-01-04, 2024-01-05.
    assert!(remaining.iter().any(|n| n.contains("2024-01-03")));
    assert!(remaining.iter().any(|n| n.contains("2024-01-04")));
    assert!(remaining.iter().any(|n| n.contains("2024-01-05")));
}

#[test]
fn test_cleanup_old_log_files_noop_when_under_limit() {
    use std::fs;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();

    for i in 0..3u32 {
        let file = path.join(format!("centy-daemon.log.2024-01-0{}", i + 1));
        fs::write(&file, "log").unwrap();
    }

    cleanup_old_log_files(path, 7);

    let count = fs::read_dir(path).unwrap().count();
    assert_eq!(count, 3);
}

#[test]
fn test_cleanup_old_log_files_ignores_non_log_files() {
    use std::fs;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();

    // Non-log file should not be counted or deleted.
    fs::write(path.join("other.txt"), "data").unwrap();
    for i in 0..5u32 {
        let file = path.join(format!("centy-daemon.log.2024-01-0{}", i + 1));
        fs::write(&file, "log").unwrap();
    }

    cleanup_old_log_files(path, 3);

    // other.txt must still be there.
    assert!(path.join("other.txt").exists());
}

#[test]
fn test_cleanup_old_log_files_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    // Should not panic on an empty directory.
    cleanup_old_log_files(dir.path(), 7);
}

#[test]
fn test_cleanup_old_log_files_nonexistent_dir() {
    let path = std::path::Path::new("/nonexistent/path/that/does/not/exist");
    // Should not panic.
    cleanup_old_log_files(path, 7);
}
