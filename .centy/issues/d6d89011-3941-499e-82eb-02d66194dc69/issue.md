# Remove all #\[allow(clippy::too_many_lines)\] directives

Remove all `#[allow(clippy::too_many_lines)]` directives scattered across the codebase by refactoring the affected functions into smaller, more maintainable pieces.

## Background

The global `too_many_lines = "allow"` was removed from Cargo.toml (issue #55). However, there are still 12 individual function-level `#[allow(clippy::too_many_lines)]` directives that bypass the lint.

## Affected Files (12 instances)

1. `src/workspace/create.rs:247`
1. `src/migration/executor.rs:34`
1. `src/server/mod.rs:162`
1. `src/server/mod.rs:2284`
1. `src/server/mod.rs:3401`
1. `src/item/entities/issue/assets.rs:270`
1. `src/item/entities/issue/create.rs:101`
1. `src/item/entities/issue/crud.rs:660`
1. `src/item/entities/issue/reconcile.rs:39`
1. `src/item/entities/pr/crud.rs:334`
1. `src/item/entities/pr/reconcile.rs:39`
1. `src/item/entities/pr/create.rs:91`

## Approach

For each function with this directive:

1. Analyze the function to understand its logical sections
1. Extract logical sections into smaller helper functions
1. Remove the `#[allow(clippy::too_many_lines)]` directive
1. Ensure tests pass after each refactoring

## Acceptance Criteria

* No `#[allow(clippy::too_many_lines)]` directives remain in the codebase
* All functions comply with Clippy’s default 100-line limit
* All existing tests continue to pass
* `cargo clippy` passes without warnings related to function length
