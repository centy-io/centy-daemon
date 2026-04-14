---
# This file is managed by Centy. Use the Centy CLI to modify it.
status: ready
createdAt: 2026-04-14T16:53:25.702581+00:00
updatedAt: 2026-04-14T16:53:25.702581+00:00
persona: developer
---

# Full test coverage enforced in CI

## User story

As a developer, I want 100% test coverage enforced in CI and pre-push hooks with no ignore exceptions, so that I can ship changes with confidence that no regressions slip through.

## Acceptance criteria

- [ ] 100% test coverage is achieved for all daemon code paths
- [ ] All per-file ignore regexes are removed from the pre-push coverage check
- [ ] Test files are renamed from numbered (01_, 02_) to descriptive names reflecting what they test
- [ ] Release process is centralized (no manual steps split across multiple locations)
- [ ] CI/CD status checks are configured and required to pass before merging
- [ ] The Makefile test targets are used in both local and CI environments

## Context

Coverage was incrementally improved from 0% to 50% and then toward 100%, with enforcement added to the pre-push hook. Some files still have per-file ignore exceptions that let coverage gaps slip through. Numbered test filenames make it hard to understand what a test covers without reading the file.

## Scope

Covers pending items in epic #418: #108, #379, #358, #85, #93
