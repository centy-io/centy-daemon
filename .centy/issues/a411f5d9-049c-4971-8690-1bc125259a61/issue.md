# Remove global dead_code lint allow

Remove the global `dead_code = "allow"` lint configuration from Cargo.toml (line 98) and address any resulting warnings properly.

## Current State

The Cargo.toml has a global lint configuration:

````toml
[lints.rust]
unsafe_code = "forbid"
# Allow dead code since many items are exported for library use
dead_code = "allow"
````

## Problem

Globally allowing dead_code hides legitimate unused code that should be cleaned up. The comment suggests this was added because “many items are exported for library use” but this is a binary crate (daemon), not a library.

## Solution

1. Remove lines 97-98 from Cargo.toml:
   
   * Remove the comment `# Allow dead code since many items are exported for library use`
   * Remove `dead_code = "allow"`
1. Run `cargo build` to identify any dead code warnings

1. For each warning, either:
   
   * Remove the unused code if it’s genuinely dead
   * Add `#[allow(dead_code)]` with a comment explaining why if it’s intentionally unused (e.g., reserved for future use)
   * Add `#[cfg(test)]` if it’s only used in tests

## Acceptance Criteria

* [ ] Global `dead_code = "allow"` removed from Cargo.toml
* [ ] No dead code warnings on `cargo build`
* [ ] Any remaining `#[allow(dead_code)]` annotations have explanatory comments
* [ ] CI passes (build + clippy)
