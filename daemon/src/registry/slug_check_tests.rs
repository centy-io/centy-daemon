use super::slug_check::{find_duplicate_slugs, warn_on_slug_conflict};
use crate::registry::types::{ProjectRegistry, TrackedProject};

fn tracked(org: Option<&str>) -> TrackedProject {
    TrackedProject {
        first_accessed: "2026-01-01T00:00:00Z".to_string(),
        last_accessed: "2026-01-01T00:00:00Z".to_string(),
        is_favorite: false,
        is_archived: false,
        organization_slug: org.map(String::from),
        user_title: None,
    }
}

#[test]
fn no_duplicates_when_different_slugs() {
    let mut registry = ProjectRegistry::new();
    registry
        .projects
        .insert("/home/user/project-a".to_string(), tracked(Some("acme")));
    registry
        .projects
        .insert("/home/user/project-b".to_string(), tracked(Some("acme")));
    let duplicates = find_duplicate_slugs(&registry);
    assert!(duplicates.is_empty());
}

#[test]
fn detects_exact_name_duplicate() {
    let mut registry = ProjectRegistry::new();
    registry
        .projects
        .insert("/home/alice/my-app".to_string(), tracked(Some("acme")));
    registry
        .projects
        .insert("/home/bob/my-app".to_string(), tracked(Some("acme")));
    let duplicates = find_duplicate_slugs(&registry);
    assert_eq!(duplicates.len(), 1);
    assert_eq!(duplicates[0].project_slug, "my-app");
    assert_eq!(duplicates[0].org_slug, "acme");
    assert_eq!(duplicates[0].project_paths.len(), 2);
}

#[test]
fn detects_slug_collision_underscore_vs_hyphen() {
    let mut registry = ProjectRegistry::new();
    registry
        .projects
        .insert("/home/alice/my_app".to_string(), tracked(Some("acme")));
    registry
        .projects
        .insert("/home/bob/my-app".to_string(), tracked(Some("acme")));
    let duplicates = find_duplicate_slugs(&registry);
    assert_eq!(duplicates.len(), 1);
    assert_eq!(duplicates[0].project_slug, "my-app");
}

#[test]
fn no_conflict_across_different_orgs() {
    let mut registry = ProjectRegistry::new();
    registry
        .projects
        .insert("/home/alice/my-app".to_string(), tracked(Some("acme")));
    registry
        .projects
        .insert("/home/bob/my-app".to_string(), tracked(Some("other")));
    let duplicates = find_duplicate_slugs(&registry);
    assert!(duplicates.is_empty());
}

#[test]
fn no_conflict_for_ungrouped_projects() {
    let mut registry = ProjectRegistry::new();
    registry
        .projects
        .insert("/home/alice/my-app".to_string(), tracked(None));
    registry
        .projects
        .insert("/home/bob/my-app".to_string(), tracked(None));
    let duplicates = find_duplicate_slugs(&registry);
    assert!(duplicates.is_empty());
}

#[test]
fn warn_on_slug_conflict_does_not_panic_on_missing_project() {
    let registry = ProjectRegistry::new();
    // Should be a no-op, not panic
    warn_on_slug_conflict(&registry, "/nonexistent");
}

#[test]
fn warn_on_slug_conflict_does_not_panic_on_no_org() {
    let mut registry = ProjectRegistry::new();
    registry
        .projects
        .insert("/home/user/app".to_string(), tracked(None));
    // Should be a no-op when project has no org
    warn_on_slug_conflict(&registry, "/home/user/app");
}
