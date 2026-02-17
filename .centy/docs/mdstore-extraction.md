---
title: "Extract Storage Layer to mdstore"
createdAt: "2026-02-17T00:13:46.686728+00:00"
updatedAt: "2026-02-17T00:13:46.686728+00:00"
---

# Extract Storage Layer to mdstore

# Extract Storage Layer to `mdstore`

## Context

The daemon’s storage engine – config-driven CRUD for Markdown files with YAML frontmatter – is a generic, reusable capability. Extracting it into a standalone `mdstore` crate (`github.com/centy-io/mdstore`) makes it usable by any project that needs file-based structured storage, not just centy.

## Scope

**mdstore** is a generic file-based storage engine. It knows nothing about `.centy/`, manifests, or assets. Functions receive a **type storage directory** path directly (e.g., `/project/.store/issues/`).

### What goes into `mdstore`

|Component|Source files|Description|
|---------|------------|-----------|
|**Frontmatter engine**|`src/common/frontmatter.rs`|Parse/generate YAML frontmatter + Markdown (title + body)|
|**Item types**|`src/item/generic/types.rs`|`GenericFrontmatter`, `GenericItem`, create/update/move/duplicate option structs|
|**CRUD engine**|`src/item/generic/storage.rs`|`create`, `get`, `list`, `update`, `delete`, `soft_delete`, `restore`, `duplicate`, `move_item`|
|**Reconciliation**|`src/item/generic/reconcile.rs`|Display number auto-increment and conflict resolution|
|**Filters**|`src/item/core/crud.rs`|`ItemFilters` (status, priority, deleted, limit, offset)|
|**Type config**|`src/config/item_type_config.rs` (partial)|`ItemTypeConfig`, `ItemTypeFeatures`, `CustomFieldDefinition` – the config schema that drives CRUD behavior|
|**Config I/O**|`src/config/item_type_config.rs` (partial)|`read_item_type_config`, `write_item_type_config`, `discover_item_types`|
|**Validation**|`src/item/validation/priority.rs`, `status.rs`|Priority/status validation, label conversion, defaults|
|**Core traits**|`src/item/core/metadata.rs`|`ItemMetadata`, `DisplayNumbered`, `Statusable`, `Prioritized`, `CustomFielded`|
|**Lifecycle traits**|`src/item/lifecycle/soft_delete.rs`|`SoftDeletable`, `Restorable`|
|**Operation traits**|`src/item/operations/*.rs`|`Movable`, `Duplicable`|
|**Item identity**|`src/item/core/id.rs`|`ItemId` (UUID/Slug), `Identifiable` trait|
|**Item trait**|`src/item/core/crud.rs`|`Item`, `ItemCrud` traits|
|**Error type**|`src/item/core/error.rs`|Unified error enum (adapted, no manifest errors)|
|**Timestamp helper**|`src/utils/mod.rs` (partial)|`now_iso()` function|
|**Metadata struct**|`src/common/metadata.rs`|`CommonMetadata` with priority migration deserialization|

### What stays in daemon

Everything centy-specific:

* `utils/` – `CENTY_FOLDER`, `CENTY_VERSION`, `get_centy_path`, `get_manifest_path`, hashing, markdown formatting
* `manifest/` – Project manifest tracking (daemon wraps CRUD calls with manifest updates)
* `config/mod.rs` – `CentyConfig`, `read_config`, `write_config`, `HookDefinition`, `CustomLinkTypeDefinition`
* `config/item_type_config.rs` – `default_issue_config`, `default_doc_config`, `migrate_to_item_type_configs`, `ItemTypeRegistry`
* `item/entities/` – Issue, Doc entity implementations
* `item/organization/` – org sync
* `common/git.rs`, `common/org_sync.rs`, `common/remote.rs`
* `server/` – gRPC layer
* `hooks/`, `link/`, `user/`, `workspace/`, `registry/`, `reconciliation/`, `template/`
* Asset file operations (copy/delete asset directories on move/duplicate/delete)

## Key API Changes (centy-daemon -> mdstore)

### Path model

Current daemon API takes `project_path` and derives storage dir internally:

````rust
generic_create(project_path: &Path, config: &ItemTypeConfig, options)
// internally: get_centy_path(project_path).join(&config.plural)
````

New mdstore API takes the **type storage directory** directly:

````rust
mdstore::create(type_dir: &Path, config: &TypeConfig, options)
// type_dir IS the storage dir (e.g., /project/.centy/issues/)
// caller resolves the path
````

### No manifest updates

Current: every CRUD operation calls `update_project_manifest(project_path)` at the end.
New: mdstore does pure file CRUD. Daemon wraps calls with manifest updates.

### No asset handling

Current: `generic_delete` and `generic_move` copy/remove asset directories.
New: mdstore only handles `.md` files. Daemon handles asset operations in its wrapper.

### Renamed types (generic, non-centy)

|Daemon name|mdstore name|
|-----------|------------|
|`GenericItem`|`Item`|
|`GenericFrontmatter`|`Frontmatter`|
|`ItemTypeConfig`|`TypeConfig`|
|`ItemTypeFeatures`|`TypeFeatures`|
|`CreateGenericItemOptions`|`CreateOptions`|
|`UpdateGenericItemOptions`|`UpdateOptions`|
|`DuplicateGenericItemOptions`|`DuplicateOptions`|
|`MoveGenericItemResult`|`MoveResult`|
|`DuplicateGenericItemResult`|`DuplicateResult`|
|`ItemError`|`StoreError`|
|`ItemFilters`|`Filters`|
|`CustomFieldDefinition`|`CustomFieldDef`|
|`generic_create`|`create`|
|`generic_get`|`get`|
|`generic_list`|`list`|
|`generic_update`|`update`|
|`generic_delete`|`delete`|
|`generic_soft_delete`|`soft_delete`|
|`generic_restore`|`restore`|
|`generic_duplicate`|`duplicate`|
|`generic_move`|`move_item`|
|`get_next_display_number_generic`|`next_display_number`|
|`reconcile_display_numbers_generic`|`reconcile_display_numbers`|

## Module Structure for `mdstore`

````
mdstore/
  Cargo.toml
  src/
    lib.rs                  # Public API re-exports
    error.rs                # StoreError enum
    frontmatter.rs          # parse_frontmatter<T>, generate_frontmatter<T>, raw variants
    metadata.rs             # CommonMetadata, deserialize_priority
    config.rs               # TypeConfig, TypeFeatures, CustomFieldDef, read/write/discover
    types.rs                # Frontmatter, Item, CreateOptions, UpdateOptions, etc.
    storage.rs              # Core CRUD: create, get, list, update, delete, soft_delete, restore, duplicate, move_item
    reconcile.rs            # next_display_number, reconcile_display_numbers
    filters.rs              # Filters struct with builder pattern
    id.rs                   # ItemId enum (UUID/Slug), Identifiable trait
    validation/
      mod.rs
      priority.rs           # validate, default, label conversion, migration
      status.rs             # status validation
    traits/
      mod.rs
      item.rs               # Item, ItemCrud traits
      metadata.rs           # ItemMetadata, DisplayNumbered, Statusable, Prioritized, CustomFielded
      lifecycle.rs           # SoftDeletable, Restorable
      operations.rs          # Movable, Duplicable
````

## Dependencies for `mdstore`

````toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
gray_matter = "0.3"
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1", features = ["full"] }
thiserror = "1"
uuid = { version = "1", features = ["v4"] }
async-trait = "0.1"
slug = "0.1.6"
tracing = "0.1"

[dev-dependencies]
tempfile = "3"
````

Note: `sha2`, `hex`, `pulldown-cmark`, `pulldown-cmark-to-cmark`, `replace-homedir`, `dirs`, `walkdir` are NOT needed – they belong to utils/hashing/formatting which stays in daemon.

## Implementation Steps

### Step 0: Fix metadata import coupling (in daemon, before extraction)

* **`src/common/metadata.rs:4`** – Change `use crate::item::entities::issue::priority::migrate_string_priority` to `use crate::item::validation::priority::migrate_string_priority`
* Same function exists in both locations; removes circular dep before extraction
* `cargo test` to verify

### Step 1: Create `mdstore` repo

* Create `github.com/centy-io/mdstore` repo
* Initialize with `cargo init --lib`
* Set up `Cargo.toml` with deps listed above
* Copy strict lint configuration from daemon
* Add `LICENSE`, basic `README.md`

### Step 2: Extract frontmatter engine

* Copy `src/common/frontmatter.rs` -> `mdstore/src/frontmatter.rs`
  * No changes needed, fully generic already
* Copy priority migration from `src/item/validation/priority.rs` -> `mdstore/src/validation/priority.rs`
* Copy `src/common/metadata.rs` -> `mdstore/src/metadata.rs`
  * Change import: `use crate::validation::priority::migrate_string_priority`
  * Replace `crate::utils::now_iso()` with inline `chrono::Utc::now().to_rfc3339()`
* Create `mdstore/src/error.rs` with `StoreError` (adapted from `ItemError`, remove `ManifestError` variant, remove `NotInitialized`/`TargetNotInitialized`)
* Verify: `cargo test` in mdstore

### Step 3: Extract config types

* Copy `ItemTypeConfig`, `ItemTypeFeatures`, `CustomFieldDefinition` from `src/config/item_type_config.rs` and `src/config/mod.rs` -> `mdstore/src/config.rs`
  * Rename to `TypeConfig`, `TypeFeatures`, `CustomFieldDef`
  * Copy `read_item_type_config`, `write_item_type_config`, `discover_item_types`
  * Adjust path resolution: functions take `type_dir: &Path` not `project_path`
  * Create `ConfigError` in mdstore (IO + JSON + YAML errors only)
* Verify: `cargo test`

### Step 4: Extract core types and traits

* Copy `src/item/core/id.rs` -> `mdstore/src/id.rs` (unchanged)
* Copy traits from `src/item/core/metadata.rs` -> `mdstore/src/traits/metadata.rs`
* Copy `Item`, `ItemCrud` from `src/item/core/crud.rs` -> `mdstore/src/traits/item.rs`
* Copy `ItemFilters` -> `mdstore/src/filters.rs` (rename to `Filters`)
* Copy lifecycle traits -> `mdstore/src/traits/lifecycle.rs`
* Copy operation traits -> `mdstore/src/traits/operations.rs`
* Copy status validation -> `mdstore/src/validation/status.rs`
* Verify: `cargo test`

### Step 5: Extract generic types and CRUD engine

* Copy `src/item/generic/types.rs` -> `mdstore/src/types.rs`
  * Rename types (GenericItem -> Item, GenericFrontmatter -> Frontmatter, etc.)
* Copy `src/item/generic/reconcile.rs` -> `mdstore/src/reconcile.rs`
  * Rename functions, adjust imports
* Copy `src/item/generic/storage.rs` -> `mdstore/src/storage.rs`
  * Rename all functions (drop `generic_` prefix)
  * Change path model: `type_storage_path(project_path, config)` -> use `type_dir` param directly
  * Remove `update_project_manifest()` calls
  * Remove asset handling from `delete` and `move_item`
  * Replace `get_centy_path()` usage with direct `type_dir` paths
  * For `move_item`: both `source_dir` and `target_dir` are passed
  * For `duplicate`: both `source_dir` and `target_dir` are passed
  * Replace `crate::utils::now_iso()` with `chrono::Utc::now().to_rfc3339()` or a local helper
  * Adjust test helper from `default_issue_config(&CentyConfig::default())` to inline `TypeConfig` construction
* Verify: `cargo test`

### Step 6: Write lib.rs public API

````rust
pub mod config;
pub mod error;
pub mod filters;
pub mod frontmatter;
pub mod id;
pub mod metadata;
pub mod reconcile;
pub mod storage;
pub mod traits;
pub mod types;
pub mod validation;

// Convenient re-exports
pub use config::{TypeConfig, TypeFeatures, CustomFieldDef, ConfigError};
pub use error::StoreError;
pub use filters::Filters;
pub use frontmatter::{parse_frontmatter, generate_frontmatter, FrontmatterError};
pub use id::{ItemId, Identifiable};
pub use metadata::CommonMetadata;
pub use storage::{create, get, list, update, delete, soft_delete, restore, duplicate, move_item};
pub use reconcile::{next_display_number, reconcile_display_numbers};
pub use types::{Item, Frontmatter, CreateOptions, UpdateOptions, DuplicateOptions, MoveResult, DuplicateResult};
````

### Step 7: Update daemon to depend on mdstore

* Add `mdstore = { git = "https://github.com/centy-io/mdstore" }` to daemon’s `Cargo.toml`
* Update `src/item/generic/storage.rs` to use `mdstore::*` internally or rewrite as thin wrappers that:
  1. Resolve `type_dir` from `get_centy_path(project_path).join(&config.plural)`
  1. Call `mdstore::create(type_dir, ...)` etc.
  1. Handle asset operations
  1. Call `update_project_manifest(project_path)`
* Update `src/item/generic/types.rs` to re-export or alias mdstore types
* Update 7 server handler files
* Update 3 test files
* Verify: `cargo test`, `cargo clippy`, E2E tests

## Function Signature Changes (storage.rs)

### Before (daemon)

````rust
pub async fn generic_create(
    project_path: &Path,
    config: &ItemTypeConfig,
    options: CreateGenericItemOptions,
) -> Result<GenericItem, ItemError>
````

### After (mdstore)

````rust
pub async fn create(
    type_dir: &Path,
    config: &TypeConfig,
    options: CreateOptions,
) -> Result<Item, StoreError>
````

### After (daemon wrapper)

````rust
pub async fn generic_create(
    project_path: &Path,
    config: &ItemTypeConfig,
    options: CreateGenericItemOptions,
) -> Result<GenericItem, ItemError> {
    let type_dir = get_centy_path(project_path).join(&config.plural);
    let item = mdstore::create(&type_dir, config, options).await?;
    update_project_manifest(project_path).await?;
    Ok(item)
}
````

## Coupling Points

|Coupling|Solution|
|--------|--------|
|`get_centy_path()` in storage functions|Caller resolves path, passes `type_dir` directly|
|`update_project_manifest()` after CRUD|Removed from lib; daemon wraps calls|
|Asset copy/delete in move/delete|Removed from lib; daemon handles assets|
|`CENTY_VERSION`|Not in lib at all|
|`common/metadata.rs` imports from `item::entities::issue`|Redirect to `item::validation::priority` before extraction|
|`ItemError` has `#[from] ManifestError`|`StoreError` doesn’t have this variant|
|`config/item_type_config.rs` has daemon-specific funcs|Only generic config types move to mdstore|
|`ItemTypeRegistry` uses `get_centy_path`|Stays in daemon (it’s centy-specific discovery logic)|

## Verification

1. `cargo test` in mdstore repo – all unit tests pass
1. `cargo clippy` in mdstore – no warnings with strict lints
1. `cargo test` in daemon – all unit + integration tests pass
1. `cargo clippy` in daemon – no warnings
1. E2E tests pass (daemon behavior unchanged)
1. Standalone mdstore test: create/read/update/delete items using only mdstore, no daemon code
