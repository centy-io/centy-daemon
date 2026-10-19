/// Folder names of the built-in item types that `centy init` creates.
///
/// Single source for the built-in set. Init, the resolve fallback and the
/// list fallback must agree with this list; the drift test in
/// `daemon/tests/builtin_item_types_drift_test.rs` enforces it.
pub const BUILTIN_ITEM_TYPE_FOLDERS: [&str; 6] = [
    "issues",
    "docs",
    "archived",
    "comments",
    "user_stories",
    "epics",
];
