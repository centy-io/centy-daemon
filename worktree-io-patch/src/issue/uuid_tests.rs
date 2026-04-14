#[test]
fn test_uuid_parse_valid() {
    assert!(uuid::Uuid::parse_str("9cad7a4b-9426-4788-9dbc-e784df999053").is_ok());
    assert!(uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").is_ok());
    assert!(uuid::Uuid::parse_str("FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF").is_ok());
}

#[test]
fn test_uuid_parse_invalid() {
    assert!(uuid::Uuid::parse_str("not-a-uuid").is_err());
    assert!(uuid::Uuid::parse_str("9cad7a4b-9426-4788-9dbc").is_err());
    assert!(uuid::Uuid::parse_str("9cad7a4b94264788-9dbc-e784df999053").is_err());
    assert!(uuid::Uuid::parse_str("9cad7a4b-9426-4788-9dbc-e784df99905z").is_err());
}
