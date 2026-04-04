# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

### Removed
- Legacy `metadata.json` folder-based issue format and all related code (`IssueMetadata` struct, `migrate.rs`, `read_issue_from_legacy_folder`, and compatibility shims)

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
