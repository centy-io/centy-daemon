---
# This file is managed by Centy. Use the Centy CLI to modify it.
status: ready
createdAt: 2026-04-14T16:53:35.391267+00:00
updatedAt: 2026-04-14T16:53:35.391267+00:00
persona: contributor
---

# Hand-rolled utilities replaced with well-maintained libraries

## User story

As a contributor, I want hand-rolled utility code replaced with well-maintained Rust crates, so that I can focus on business logic rather than maintaining low-level utilities that already exist as proven libraries.

## Acceptance criteria

- [ ] Custom git remote URL parsing is replaced by the git-url-parse crate
- [ ] Custom frontmatter parsing is replaced by a library (e.g. gray_matter or similar)
- [ ] tracing-error is added for coloured error output in logs
- [ ] Atomic file operations use the tempfile crate instead of hand-rolled temp-file logic
- [ ] All replaced utilities are removed from the codebase with no dead code remaining
- [ ] The tracking issue #75 (Replace logic with lib) reflects the completed state

## Context

Multiple hand-rolled utilities duplicate functionality available in maintained crates. These were initially written to avoid dependencies but now create maintenance burden. Previous replacements (humantime, tower-http, git2, serde_json merge) proved the pattern works well for this codebase.

## Scope

Covers pending items in epic #419: #75, #266, #199, #114, #107
