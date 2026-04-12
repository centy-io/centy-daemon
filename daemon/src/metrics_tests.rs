use super::*;

#[test]
fn test_operation_timer_creation() {
    let timer = OperationTimer::new("test_op");
    assert_eq!(timer.name, "test_op");
}

#[test]
fn test_operation_timer_drop_logs() {
    // Just verify it doesn't panic when dropped
    let _timer = OperationTimer::new("test_drop");
    // Timer will be dropped at end of scope
}

#[test]
fn test_generate_request_id_format() {
    let id = generate_request_id();
    assert_eq!(id.len(), 8);
}

#[test]
fn test_generate_request_id_unique() {
    let id1 = generate_request_id();
    let id2 = generate_request_id();
    assert_ne!(id1, id2);
}

#[test]
fn test_generate_request_id_hex_chars() {
    let id = generate_request_id();
    // cspell:ignore hexdigit
    assert!(id.chars().all(|c| c.is_ascii_hexdigit() || c == '-'));
}
