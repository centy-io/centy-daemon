# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Issue #418: `worktree open centy:<uuid>` now works — `parse_centy` accepts both display numbers (`centy:42`) and internal UUIDs (`centy:6f4853a9-…`); UUIDs are resolved to their display number via the matching `.centy/issues/<uuid>.md` file
- `resolve_issue` in the daemon now strips an optional `centy:` prefix before resolving, so callers may pass `centy:42`, `centy:<uuid>`, or bare IDs interchangeably

### Added
- `gap-analyze` skill: inspects an epic's stated goals, creates or updates user stories to cover any gaps, and raises issues for stories that fail the quality bar (missing body, user-story statement, acceptance criteria, or non-draft status)
- `future-man` skill: analyzes the project (codebase structure, docs, existing Centy items) and seeds a custom `ideas` item type with forward-thinking, context-grounded ideas spanning quick wins to moonshots; ideas carry structured metadata — `category`, `horizon`, `impact`, and `inspiration` — and flow through a `raw → promising → validated → shelved` lifecycle
- Issue #416: `epic` as a built-in default item type with no `status` field (statusless)

## [0.13.0] — 2026-04-13

### Added
- Cascade hard-delete of link items when a linked item is deleted: all link records referencing
  the deleted item (as source or target) are now hard-deleted atomically in the same operation
- Orphan link sweep in the regular cleanup pass: pre-existing orphan links are hard-deleted at
  the end of each cleanup cycle
- `cascade_delete_entity_links` — public API to remove all links for an entity
- `list_all_links` — public API to list every link record in the project
- `clean_orphan_links_for_project` — public API for an on-demand orphan sweep

## [0.12.4] — 2026-04-13

### Changed
- User stories no longer have default statuses — the `user_stories` item type is now status-free out of the box

## [0.12.3] — 2026-04-13

### Changed
- `compact` skill now auto-discovers issues when called with no arguments — groups them into feature clusters and runs the full compact workflow without requiring any user input
- Compacted 113 closed lint/code-quality issues into new epic #413 (Rust Lint & Code Quality Hardening); all soft-deleted
- Updated epic #1 (Generic & Config-Driven Item Type System) with progress summary; linked 19 active child issues; soft-deleted 4 completed issues

### Added
- `user_stories` is now a built-in default item type seeded automatically on project init, giving teams an out-of-the-box agile workflow type alongside issues and docs
- Issue #413: Show linkage in all item types — `GetItem` / `ListItems` / `SearchItems` should return inline `links` data with direction, type, and resolved title; all item types must be first-class link participants
- Claude Code plugin marketplace scaffold: users can now install the `centy` plugin via `/plugin marketplace add centy-io/centy-daemon`
- `plugins/centy/skills/install/SKILL.md` — guided install skill (clone → build → start daemon → register MCP server)
- `plugins/centy/skills/mcp-usage/SKILL.md` — MCP tools reference skill covering daemon lifecycle, CRUD operations, query language, and common workflows
- `plugins/centy/skills/compact/SKILL.md` — compact skill that consolidates related issues into a feature item (epic), links active issues as children, and soft-deletes done issues; steps 1–4 run in a research-only sub-agent
- `plugins/centy/.mcp.json` — auto-wires `centy-mcp` when the plugin is installed
- `.github/workflows/validate-plugins.yml` — CI workflow that validates plugin JSON manifests and SKILL.md presence
- `.claude-plugin/README.md` — documents the marketplace layout, installation from another repo, and skill invocation syntax (`/centy:<skill>`)

### Fixed
- `plugins/centy/.claude-plugin/plugin.json` — removed invalid `skills` array (skills are auto-discovered; the field is not part of the manifest schema)
- All three `SKILL.md` files now include required YAML frontmatter (`name`, `description`, `version`) so the plugin validator can parse skill metadata

## [0.12.2] — 2026-04-13

### Added
- gRPC Health Checking Protocol support (`grpc.health.v1`) via `tonic-health`: the daemon now exposes a standard `Health` service (Check + Watch) that reports `SERVING` on startup and `NOT_SERVING` on shutdown/restart, enabling `grpc_health_probe`, load balancers, and orchestrators to query daemon liveness without custom logic
- `release.yml` now builds and uploads `centy-cli` binaries for all 5 platforms (linux-x86_64, linux-aarch64, darwin-x86_64, darwin-aarch64, windows-x86_64) as part of the GitHub release
- Root `Makefile` with a `build` target that builds all components (`cargo build --release` for the daemon, `cli/Makefile`, and `mcp/Makefile`)
- `centy-mcp` now checks version compatibility with the daemon on startup and emits a clear error if they are incompatible (e.g. `centy-mcp v0.9.2 is incompatible with centy-daemon v0.10.5. Please update centy-mcp.`)
- New `cli/` module: auto-generated gRPC CLI (`centy`) built from proto definitions via `protoc-gen-cobra`, exposing all 80+ RPCs as `centy <rpc>` subcommands with full flag and JSON I/O support
- `cli/README.md` documenting installation, usage, and regeneration workflow
- `test` and `coverage` targets to the root `Makefile`; the pre-push hook now delegates to `make test` and `make coverage` instead of calling `cargo` directly

### Changed
- CLI RPCs are now invocable as `centy <rpc>` instead of `centy centy-daemon <rpc>`; the redundant `centy-daemon` subcommand level has been removed
- Pre-commit hook now uses `lint-staged` to run `cspell` only on staged files, replacing the manual `git diff --cached` + `xargs` approach
- Extracted `lint-staged` configuration from `package.json` into a dedicated `.lintstagedrc.mjs` file
- Updated `CONTRIBUTING.md` to reflect the current project state: corrected project structure tree (`daemon/src/`), fixed integration test import path, updated Rust prerequisite to 1.85+, and revised the "Adding a New Feature" gRPC workflow

## [0.12.1] — 2026-04-12

### Changed
- Moved all daemon-specific source code, tests, packaging, and tooling into a `daemon/` sub-folder, mirroring the `mcp/` layout

## [0.12.0] — 2026-04-04

### Added
- `UpdateLink` gRPC endpoint for changing the link type of an existing link
- `org_wide` flag on `CreateItem`: when set, writes the item to the org-wide `.centy` repo and tags it with the originating project's slug via `projects` metadata field
- `find_org_repo` registry helper: discovers the org-wide repo for a project by scanning tracked projects in the same org whose path ends with `/.centy`
- `projects` field on `GenericItemMetadata` proto (surfaces org project associations)
- `include_organization_items` field on `ListItemsRequest` proto (stub for upcoming org-wide list support)
- `projects` field on `CreateItemRequest` proto (stub for upcoming multi-project association)
- `projects: Vec<String>` field on `IssueFrontmatter` for proper roundtrip through issue-specific code paths (reconcile, CRUD, move)
- `ListItems` now merges org-wide items via `include_organization_items` flag (default `true`); org items are filtered by the current project's slug and carry `source: "org"` while project-local items carry `source: "project"`
- `GetItem` falls back to the org repo when the requested item is not found in the project's own `.centy/`; returned item carries `source: "org"` on both `GenericItem` and `GetItemResponse`
- `UpdateItem` and `DeleteItem` transparently route writes to the org repo when the target item lives there; project-local items are unaffected; display-number resolution also falls back to the org repo

### Changed
- Split `init/mcp_json.rs`: pure JSON logic stays in `mcp_json.rs`; async file I/O extracted into new `mcp_io.rs`
- Refactored `workspace_temp` handler into focused modules: `hooks.rs` (status update), `operations/create.rs` (workspace creation), `operations/editor.rs` (editor invocation), with `handler.rs` as thin orchestration
- Split `link/storage/io.rs` into `io.rs` (file operations), `serialization.rs` (deserialization logic), and `validation.rs` (link validation rules)

### Removed
- Legacy `metadata.json` folder-based issue format and all related code (`IssueMetadata` struct, `migrate.rs`, `read_issue_from_legacy_folder`, and compatibility shims)
- `source` field from `GenericItem` and `GetItemResponse`, and `org_wide` flag from `CreateItemRequest` (V1 org-wide design leftovers; V2 routes writes transparently so clients no longer need these)
- `clear_projects` field from `UpdateItemRequest` proto (not yet supported in mdstore)
- `projects` field from `UpdateOptions` (not yet supported in mdstore)

## [0.11.1] — 2026-04-04

### Changed
- Replace custom `is_valid_plural` implementation with `slug` crate

## [0.11.0] — 2026-04-04

### Removed
- `GetReconciliationPlan` and `ExecuteReconciliation` gRPC endpoints; reconciliation is now internal-only and `Init` is the sole public entry point

### Changed
- Replace hand-rolled `merge_json_content` JSON merge logic with `json-patch` crate (RFC 7396 JSON Merge Patch)

## [0.10.5] — 2026-04-03

### Changed
- Upgrade npm to latest in CI for OIDC trusted publishing support (requires npm >= 11.5.1)

## [0.10.4] — 2026-04-03

### Changed
- Restore registry-url in setup-node for npm OIDC auth to work

## [0.10.3] — 2026-04-03

### Changed
- Fix npm OIDC publishing by removing registry-url from setup-node

## [0.10.2] — 2026-04-03

### Changed
- Consolidate MCP release into main release workflow; delete `mcp-release.yml`

## [0.10.1] — 2026-04-02

### Fixed
- Prevent org inference for git subdirectories: `infer_organization_from_remote` now requires the path to be the git root, so `organization.json` is not written into subdirectory `.centy` dirs

## [0.10.0] — 2026-04-02

### Added
- Generate `.mcp.json` during `init`: create with centy MCP entry when absent, inject into existing file preserving other servers, no-op if already present, abort on invalid JSON (#381)
- Add `listed` flag to `ItemTypeConfig` to control visibility in `ListItemTypes`; `comments` and `archived` default to `listed: false`

### Removed
- Remove PR as a built-in entity type; users can define custom `pr` item types via `config.yaml` (#368)

## [0.9.3] — 2026-04-02

### Changed
- Remove legacy `LinkTargetType` enum from proto; use string-based `*_item_type` fields directly in link RPCs (#367)
- Migrate hooks from `config.json` to `hooks.yaml` as the single source of truth (#362)
- Rename hook patterns to event-driven convention: `item_type.event` (e.g. `issue.creating`, `*.deleted`, `*.*`) replacing the old `phase:type:operation` format
- Remove `hooks` field from gRPC `Config` proto (reserve field 12)

## [0.9.0] — 2026-03-24

### Added
- Tags support for all item types with MQL `$in`/`$all` filter operators (#357)

### Changed
- Switch mdstore to published 1.1.0

## [0.8.4] — 2026-03-24

### Added
- Comments as a built-in item type with `item_id`/`item_type`/`author` custom fields (#356)
- MQL `customFields` filter for post-retrieval filtering by custom field value

### Changed
- Remove deprecated `EditorType` enum and `editor_type` field from workspace RPCs
- Update worktree-io dependency to 0.17.4

### Fixed
- Vendor OpenSSL in git2 to fix CI builds on Linux/macOS cross-compilation
- Prevent cspell from failing on lock files and non-text staged files

## [0.8.3] — 2026-03-19

### Changed
- Update worktree-io dependency to 0.17.1

## [0.8.2] — 2026-03-16

### Added
- User-defined free-form key-value pairs to project config
- Stub `ListItemsAcrossProjects` RPC for cross-project item queries (#354)
- Release new version routine with automated steps

### Changed
- Replace custom git URL parsing with `git-url-parse` crate (#266)
- Replace git subprocess calls with `git2` crate for branch and remote operations
- Replace inline proto definitions with git submodule
- Replace custom duration parser with `humantime` crate (#203)
- Replace custom `GrpcLoggingLayer` with `tower-http` `TraceLayer` (#204)
- Upgrade mdstore to 1.0.0 with native frontmatter comment injection (#259)
- Update worktree-io dependency to 0.15.0 (#180)
- Remove legacy `allowedStates` from `CentyConfig` (#202)
- Remove `defaultStatus` from item type config (#182)
- Remove deprecated `issue_number` field and backward-compat functions
- Remove `updatedAt` from manifest
- Enforce 100-line file size limit for Rust source files (#177)
- Comprehensive clippy lint campaign: deny 880+ lints including full lint groups (correctness, style, suspicious, complexity, perf) and 50+ Rust standard lints
- Split oversized handler files into modules to comply with line limits (#324–#344)

### Fixed
- Generic item create/update/soft-delete/restore/duplicate/move missing managed-by header (#258)
- Skip tracking projects in ignored/temp directories (#205)
- Remove implicit `.expect()` in main by replacing `#[tokio::main]` (#345)
- Add submodule checkout to release workflow

## [0.8.1] — 2026-03-05

### Added
- Auto hard-delete artifacts after configurable retention period (#257)
- Auto-initialize `hooks.yaml` on project init (#170)
- Worktree configuration file for shared settings

### Changed
- Remove custom doc item type logic (doc now treated as generic item type)
- Lint suppression cleanup: removed all `#[allow]` directives across the codebase (#216–#253)
- Remove `features.status` field from item type config; status is now derived from `statuses` list (#255)

### Fixed
- Await `track_project` in init handler to prevent race with `getProjectInfo` (#171)
- Poisoned mutex recovery in registry tests (#167)
- Isolate integration tests via `CENTY_HOME` to prevent cross-binary registry races (#162)

## [0.8.0] — 2026-02-22

### Added
- Organization-level issues gRPC RPCs and storage (#87)
- `ListItemTypes` gRPC endpoint with integration and e2e tests (#203)
- Managed-by header comment on all daemon-managed files (#214)
- Default archived item folder with `original_item_type` custom field (#165)
- Expose project version and behind-remote status in `ProjectInfo` (#70)
- Cascade flag for `DeleteOrganization` RPC (#206)
- Configurable ignore paths in user global config (#204)
- All config options now configurable during project initialization (#86)

### Changed
- Replace all per-entity gRPC RPCs with generic item API (#191)
- Update worktree-io dependency from 0.9.0 to 0.14.0 (#151)
- Use YAML comment inside frontmatter instead of HTML comment before it

### Fixed
- Show log file path on all daemon startup failures (#153)
- Structured error in `close_temp_workspace` handler
- `module_inception` lint in `doc/tests.rs`

## [0.7.0] — 2026-02-17

### Added
- MQL filter support in `ListItems` RPC
- User global config at `~/.config/centy/config.toml` (#83)
- `GetItem` feature parity with all legacy entity-specific Get RPCs (#200)
- Ad-hoc codesign for macOS release binaries

### Changed
- Replace custom workspace logic with worktree-io integration (#82)
- Use mdstore 0.4.0 as dependency

## [0.6.0] — 2026-02-15

### Added
- Generic CRUD gRPC RPCs for all item types (#190)
- `CreateItemType` gRPC endpoint for dynamic item type creation (#66)
- Generic item move via `generic_move()` across all item types (#70)
- Generic duplicate logic for all item types (static and custom)
- `ItemTypeConfig` schema with validation for `config.yaml` (#171)
- Assert service to enforce preconditions before each command (#188)
- Merge `cspell.json` on init instead of overwriting (#187)

### Changed
- Replace hardcoded `ItemType` enums with dynamic registry lookup (#176)
- Eliminate entity-specific delete handler files (#194)
- Extract search logic from daemon into dedicated module (#163)
- Replace custom frontmatter parsing with `gray_matter` crate (#199)
- Remove legacy `allowedStates` from `CentyConfig` and gRPC API (#78)
- Extract `mdstore` as a reusable library dependency (#79)
- Promote `clippy::too_many_lines` from warn to deny

### Fixed
- Soft delete is now always enabled — removed `softDelete` config option (#189)
- Remove `defaultState` from config (#192)

## [0.5.0] — 2026-02-11

### Added
- Migration to insert `config.yaml` into existing projects (#174)
- Generic storage layer for config-driven item types (#177)

### Changed
- Remove all LLM/agent management code (`LlmConfig`, llm module) (#132)
- Remove PR entity type and all references from codebase (#184)
- Delegate deprecated workspace RPCs to unified handlers (#134)

## [0.4.0] — 2026-02-06

### Added
- Structured JSON error responses replacing plain-string gRPC errors (#161)
- Errors returned in response body instead of gRPC status codes (#129)
- Persistent hook execution history with gRPC API
- `ConfigService` extracted from `CentyDaemon` to decompose monolithic service
- Auto-add hooks section to `config.json` for existing projects (#157)
- Flat dot-separated config keys (VS Code style) (#159)
- `clippy::arithmetic_side_effects` lint enabled (#145)
- Panic/unwrap Clippy lints promoted from warn to deny

### Changed
- Split `server/mod.rs` (5,949 lines) into ~80 submodule files (#118)
- Increased unit test coverage across 20 modules

### Fixed
- Proper shell escaping in terminal and editor command construction (#146)
- Replace `std::process::exit(1)` with graceful error propagation (#33)

## [0.3.1] — 2026-02-02

### Added
- Custom lifecycle hooks system (#137)
- Show log file location when daemon fails to start
- gRPC API versioned under `centy.v1` package

### Fixed
- Resolve display numbers to UUIDs in all issue/PR RPC handlers
- Extract `LOG_FILENAME` constant for consistent log path reporting

## [0.2.0] — 2025-12-23

### Added
- Organization-wide documentation support with cross-project sync
- Standalone workspace support (#22)
- Terminal workspace support and `GetSupportedEditors` RPC
- Soft delete and restore RPCs for all entities
- Docker e2e testing infrastructure (#100)
- Auto sync centy branch (#99)
- Stdin prompt support for terminal agents
- Pre-push hook with lint and build checks (#10)
- Colored error output via `tracing-error` and `color-eyre` (#12)
- Configurable editor support with unified workspace RPCs
- Cross-organization issue support

### Changed
- Migrate issue/PR metadata from JSON to YAML frontmatter Markdown format
- Consolidate issue, PR, and doc into unified item domain (DDD)
- Consolidate shared fields between `IssueMetadata` and `PrMetadata`
- Replace custom SemVer parsing with `semver` crate

[Unreleased]: https://github.com/centy-io/centy-daemon/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/centy-io/centy-daemon/compare/v0.8.4...v0.9.0
[0.8.4]: https://github.com/centy-io/centy-daemon/compare/v0.8.3...v0.8.4
[0.8.3]: https://github.com/centy-io/centy-daemon/compare/v0.8.2...v0.8.3
[0.8.2]: https://github.com/centy-io/centy-daemon/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/centy-io/centy-daemon/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/centy-io/centy-daemon/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/centy-io/centy-daemon/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/centy-io/centy-daemon/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/centy-io/centy-daemon/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/centy-io/centy-daemon/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/centy-io/centy-daemon/compare/v0.2.0...v0.3.1
[0.2.0]: https://github.com/centy-io/centy-daemon/releases/tag/v0.2.0
