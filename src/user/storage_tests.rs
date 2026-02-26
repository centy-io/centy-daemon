use super::*;

fn make_user(id: &str, name: &str, email: Option<&str>) -> User {
    User {
        id: id.to_string(),
        name: name.to_string(),
        email: email.map(ToString::to_string),
        git_usernames: vec![],
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
        deleted_at: None,
    }
}

#[test]
fn test_find_user_by_email_found() {
    let users = vec![
        make_user("alice", "Alice", Some("alice@example.com")),
        make_user("bob", "Bob", Some("bob@example.com")),
    ];

    let found = find_user_by_email(&users, "alice@example.com");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, "alice");
}

#[test]
fn test_find_user_by_email_not_found() {
    let users = vec![make_user("alice", "Alice", Some("alice@example.com"))];

    let found = find_user_by_email(&users, "unknown@example.com");
    assert!(found.is_none());
}

#[test]
fn test_find_user_by_email_none_emails() {
    let users = vec![make_user("alice", "Alice", None)];

    let found = find_user_by_email(&users, "alice@example.com");
    assert!(found.is_none());
}

#[test]
fn test_find_user_by_email_empty_list() {
    let users: Vec<User> = vec![];
    let found = find_user_by_email(&users, "test@example.com");
    assert!(found.is_none());
}

#[test]
fn test_find_user_by_id_found() {
    let users = vec![
        make_user("alice", "Alice", None),
        make_user("bob", "Bob", None),
    ];

    let found = find_user_by_id(&users, "bob");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Bob");
}

#[test]
fn test_find_user_by_id_not_found() {
    let users = vec![make_user("alice", "Alice", None)];

    let found = find_user_by_id(&users, "unknown");
    assert!(found.is_none());
}

#[test]
fn test_find_user_by_id_empty_list() {
    let users: Vec<User> = vec![];
    let found = find_user_by_id(&users, "test");
    assert!(found.is_none());
}
