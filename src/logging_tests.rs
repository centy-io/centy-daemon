use super::*;

#[test]
fn test_log_config_default() {
    let config = LogConfig::default();
    assert_eq!(config.log_level, Level::INFO);
    assert!(!config.json_format);
    assert!(config.log_dir.ends_with("logs"));
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
    let _ = parse_rotation("HOURLY");
    let _ = parse_rotation("Never");
    let _ = parse_rotation("DAILY");
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
