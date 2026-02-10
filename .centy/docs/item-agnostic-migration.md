---
title: "Item-Agnostic Migration Spec"
createdAt: "2026-02-10T23:12:32.009378+00:00"
updatedAt: "2026-02-10T23:12:32.009378+00:00"
---

# Item-Agnostic Migration Spec

This document specifies the migration from hardcoded item types (Issue, PR, Doc) to fully config-defined item types in centy-daemon v2.

## 1. Motivation

Today the daemon has three hardcoded item types: `issue`, `pr`, and `doc`. Every new item type requires changes across the proto file, Rust entity modules, gRPC handlers, config, templates, search, links, org-sync, and reconciliation. This coupling makes the system rigid and prevents users from defining their own workflows (epics, tasks, stories, RFCs, bugs, spikes, etc.).

The goal is to make item types fully defined in `config.json` so that:

* Users can create arbitrary item types without daemon code changes.
* The gRPC API is a single set of generic RPCs instead of per-type RPCs.
* The Rust domain layer operates on a single generic `Item` type driven by schema configuration.
* Existing `issue`/`pr`/`doc` projects migrate seamlessly.

## 2. Config Schema (v2)

### 2.1 Top-Level Config

The new `config.json` introduces an `itemTypes` map. Each key is the item type slug, each value is its schema definition.

````json
{
  "version": "0.2.0",
  "itemTypes": {
    "issue": { "...": "see 2.2" },
    "pr": { "...": "see 2.2" },
    "doc": { "...": "see 2.2" }
  },
  "llm": { "...": "unchanged" },
  "customLinkTypes": [],
  "defaultEditor": "vscode",
  "hooks": []
}
````

Global fields that were previously at the top level (`priorityLevels`, `allowedStates`, `defaultState`, `stateColors`, `priorityColors`, `customFields`, `defaults`) move **inside each item type definition**. They no longer exist at the top level.

### 2.2 Item Type Definition

````json
{
  "issue": {
    "folder": "issues",
    "identifier": "uuid",
    "features": {
      "displayNumber": true,
      "status": true,
      "priority": true,
      "softDelete": true,
      "assets": true,
      "orgSync": true,
      "move": true,
      "duplicate": true
    },
    "allowedStates": ["open", "in-progress", "closed", "testing"],
    "defaultState": "open",
    "stateColors": {
      "open": "#10b981",
      "in-progress": "#f59e0b",
      "closed": "#6b7280"
    },
    "priorityLevels": 3,
    "priorityColors": {},
    "fields": {
      "draft": { "type": "bool", "default": false },
      "assignee": { "type": "string" }
    },
    "defaults": {}
  }
}
````

````json
{
  "pr": {
    "folder": "prs",
    "identifier": "uuid",
    "features": {
      "displayNumber": true,
      "status": true,
      "priority": true,
      "softDelete": true,
      "assets": false,
      "orgSync": false,
      "move": false,
      "duplicate": false
    },
    "allowedStates": ["draft", "open", "merged", "closed"],
    "defaultState": "draft",
    "stateColors": {},
    "priorityLevels": 3,
    "priorityColors": {},
    "fields": {
      "source_branch": { "type": "string", "required": true },
      "target_branch": { "type": "string", "required": true },
      "reviewers": { "type": "string[]", "default": [] },
      "merged_at": { "type": "datetime" },
      "closed_at": { "type": "datetime" }
    },
    "defaults": {
      "target_branch": "main"
    }
  }
}
````

````json
{
  "doc": {
    "folder": "docs",
    "identifier": "slug",
    "features": {
      "displayNumber": false,
      "status": false,
      "priority": false,
      "softDelete": true,
      "assets": false,
      "orgSync": true,
      "move": true,
      "duplicate": true
    },
    "allowedStates": [],
    "fields": {},
    "defaults": {}
  }
}
````

User-defined types work the same way:

````json
{
  "epic": {
    "folder": "epics",
    "identifier": "uuid",
    "features": {
      "displayNumber": true,
      "status": true,
      "priority": true,
      "softDelete": true,
      "assets": false,
      "orgSync": false,
      "move": true,
      "duplicate": true
    },
    "allowedStates": ["planning", "active", "done"],
    "defaultState": "planning",
    "priorityLevels": 3,
    "fields": {
      "target_date": { "type": "date" },
      "owner": { "type": "string" }
    }
  }
}
````

### 2.3 Field Type System

Supported field types for the `fields` map:

|Type|Description|Frontmatter representation|
|----|-----------|--------------------------|
|`string`|Free-form text|`fieldName: "value"`|
|`string[]`|List of strings|`fieldName: ["a", "b"]`|
|`number`|Integer or float|`fieldName: 42`|
|`bool`|Boolean|`fieldName: true`|
|`date`|ISO date (YYYY-MM-DD)|`fieldName: "2025-03-15"`|
|`datetime`|ISO timestamp|`fieldName: "2025-03-15T10:30:00Z"`|
|`enum`|Constrained string|`fieldName: "value"`|

Enum fields include an `enumValues` array:

````json
{
  "severity": {
    "type": "enum",
    "enumValues": ["critical", "major", "minor", "trivial"],
    "default": "minor"
  }
}
````

Field properties:

* `type` (required): One of the types above.
* `required` (optional, default `false`): Validation fails if missing on create.
* `default` (optional): Default value applied on create if not provided.
* `enumValues` (required for `enum` type): Allowed values.

### 2.4 Feature Flags

The `features` object determines which daemon capabilities are enabled for this item type:

|Feature|Description|
|-------|-----------|
|`displayNumber`|Sequential human-readable numbering (1, 2, 3…)|
|`status`|Status field with allowed-states validation|
|`priority`|Priority field with configurable levels|
|`softDelete`|Recoverable deletion via `deleted_at` timestamp|
|`assets`|File attachments on items|
|`orgSync`|Sync items across organization projects|
|`move`|Move items between projects|
|`duplicate`|Duplicate items within/across projects|

### 2.5 Identifier Mode

* `"uuid"` – Items stored as `.centy/{folder}/{uuid}.md`. Supports display numbers.
* `"slug"` – Items stored as `.centy/{folder}/{slug}.md`. Title-derived, like current docs.

## 3. gRPC API (v2)

### 3.1 Unified Item RPCs

The current ~37 type-specific item RPCs collapse into a single generic set:

````protobuf
service CentyDaemon {
  // Item CRUD
  rpc CreateItem(CreateItemRequest) returns (CreateItemResponse);
  rpc GetItem(GetItemRequest) returns (GetItemResponse);
  rpc GetItemByDisplayNumber(GetItemByDisplayNumberRequest) returns (GetItemResponse);
  rpc GetItemsByIdentifier(GetItemsByIdentifierRequest) returns (GetItemsByIdentifierResponse);
  rpc ListItems(ListItemsRequest) returns (ListItemsResponse);
  rpc UpdateItem(UpdateItemRequest) returns (UpdateItemResponse);
  rpc DeleteItem(DeleteItemRequest) returns (DeleteItemResponse);
  rpc SoftDeleteItem(SoftDeleteItemRequest) returns (SoftDeleteItemResponse);
  rpc RestoreItem(RestoreItemRequest) returns (RestoreItemResponse);
  rpc MoveItem(MoveItemRequest) returns (MoveItemResponse);
  rpc DuplicateItem(DuplicateItemRequest) returns (DuplicateItemResponse);
  rpc GetNextItemNumber(GetNextItemNumberRequest) returns (GetNextItemNumberResponse);

  // Search
  rpc AdvancedSearch(AdvancedSearchRequest) returns (AdvancedSearchResponse);

  // Item type introspection
  rpc GetItemTypes(GetItemTypesRequest) returns (GetItemTypesResponse);
  rpc GetItemTypeSchema(GetItemTypeSchemaRequest) returns (ItemTypeSchema);

  // ... (all non-item RPCs remain unchanged)
}
````

### 3.2 Generic Item Message

````protobuf
message Item {
  string id = 1;                          // UUID or slug depending on identifier mode
  string item_type = 2;                   // e.g. "issue", "pr", "epic"
  uint32 display_number = 3;              // 0 if displayNumber feature is off
  string title = 4;
  string body = 5;
  string status = 6;                      // empty if status feature is off
  uint32 priority = 7;                    // 0 if priority feature is off
  string priority_label = 8;
  string created_at = 9;
  string updated_at = 10;
  string deleted_at = 11;                 // empty if not soft-deleted
  map<string, FieldValue> fields = 12;    // all type-specific fields
  bool is_org_item = 13;
  string org_slug = 14;
  uint32 org_display_number = 15;
}

message FieldValue {
  oneof value {
    string string_value = 1;
    int64 number_value = 2;
    bool bool_value = 3;
    StringList list_value = 4;
  }
}

message StringList {
  repeated string values = 1;
}
````

### 3.3 Generic Request Messages

````protobuf
message CreateItemRequest {
  string project_path = 1;
  string item_type = 2;                   // required: which type to create
  string title = 3;
  string body = 4;
  string status = 5;                      // optional, uses defaultState if empty
  uint32 priority = 6;                    // optional, uses default if 0
  map<string, string> fields = 7;         // type-specific fields
  string template = 8;
  bool is_org_item = 9;
  string slug = 10;                       // only for slug-identified types
}

message ListItemsRequest {
  string project_path = 1;
  string item_type = 2;                   // required: which type to list
  string status = 3;
  uint32 priority = 4;
  bool include_deleted = 5;
  map<string, string> field_filters = 6;  // filter on type-specific fields
}

message GetItemRequest {
  string project_path = 1;
  string item_type = 2;
  string item_id = 3;                     // UUID or slug
}

message GetItemByDisplayNumberRequest {
  string project_path = 1;
  string item_type = 2;
  uint32 display_number = 3;
}

message UpdateItemRequest {
  string project_path = 1;
  string item_type = 2;
  string item_id = 3;
  string title = 4;
  string body = 5;
  string status = 6;
  uint32 priority = 7;
  map<string, string> fields = 8;
  string new_slug = 9;                    // only for slug-identified types
}

message MoveItemRequest {
  string source_project_path = 1;
  string item_type = 2;
  string item_id = 3;
  string target_project_path = 4;
  string new_slug = 5;                    // only for slug-identified types
}
````

### 3.4 Backward Compatibility

Two options:

**Option A – Proto v2 package:** Introduce `centy.v2` package. Keep `centy.v1` running in parallel with a compatibility shim that translates v1 calls into v2. Deprecate v1 over a release cycle.

**Option B – In-place migration:** Replace v1 RPCs with v2 RPCs in a single major version bump. Clients must update. Simpler daemon, harder on clients.

Recommendation: **Option A** – run both in parallel. The v1 shim is thin (just maps `CreateIssue` to `CreateItem{item_type: "issue"}`) and lets existing CLI versions continue working.

### 3.5 Item Type Introspection RPCs

````protobuf
message GetItemTypesRequest {
  string project_path = 1;
}

message GetItemTypesResponse {
  repeated ItemTypeInfo item_types = 1;
  bool success = 2;
  string error = 3;
}

message ItemTypeInfo {
  string name = 1;                        // e.g. "issue", "pr", "epic"
  string folder = 2;
  string identifier = 3;                  // "uuid" or "slug"
  ItemTypeFeatures features = 4;
  repeated string allowed_states = 5;
  string default_state = 6;
  uint32 priority_levels = 7;
  repeated FieldDefinition fields = 8;
}

message ItemTypeFeatures {
  bool display_number = 1;
  bool status = 2;
  bool priority = 3;
  bool soft_delete = 4;
  bool assets = 5;
  bool org_sync = 6;
  bool move = 7;
  bool duplicate = 8;
}

message FieldDefinition {
  string name = 1;
  string field_type = 2;                  // "string", "number", "bool", etc.
  bool required = 3;
  string default_value = 4;
  repeated string enum_values = 5;
}
````

These replace the need for clients to parse `config.json` directly and enable dynamic UI generation.

## 4. Storage

### 4.1 Directory Structure

````
.centy/
  config.json             # v2 config with itemTypes
  .centy-manifest.json
  project.json
  users.json
  organization.json
  templates/
    issue/default.md
    pr/default.md
    epic/default.md       # user-defined type templates
  assets/                 # shared assets (unchanged)
  issues/                 # folder name from config: itemTypes.issue.folder
    {uuid}.md
  prs/                    # folder name from config: itemTypes.pr.folder
    {uuid}.md
  docs/                   # folder name from config: itemTypes.doc.folder
    {slug}.md
  epics/                  # user-defined type
    {uuid}.md
````

Folder names come from `itemTypes.{type}.folder` in config. This means existing `.centy/issues/` and `.centy/prs/` directories continue to work without renaming.

### 4.2 Frontmatter Format

All item types use the same YAML frontmatter structure, with type-specific fields appearing as additional keys:

**UUID-identified item (e.g., issue):**

````yaml
---
displayNumber: 5
status: open
priority: 1
createdAt: "2025-12-02T21:27:50Z"
updatedAt: "2025-12-03T07:35:41Z"
draft: false
assignee: "john"
---
````

**Slug-identified item (e.g., doc):**

````yaml
---
title: "Getting Started"
createdAt: "2025-12-02T21:27:50Z"
updatedAt: "2025-12-03T07:35:41Z"
---
````

The frontmatter parser becomes generic: it reads the item type’s schema from config and parses/validates fields accordingly.

## 5. Rust Domain Layer

### 5.1 Replace `ItemType` Enum

The compile-time enum in `src/item/mod.rs`:

````rust
pub enum ItemType {
    Issue,
    PullRequest,
    Doc,
}
````

Becomes a runtime string validated against config:

````rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemType(String);

impl ItemType {
    pub fn new(name: &str, config: &CentyConfig) -> Result<Self, ItemError> {
        if config.item_types.contains_key(name) {
            Ok(Self(name.to_string()))
        } else {
            Err(ItemError::UnknownItemType(name.to_string()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
````

### 5.2 Generic Item Struct

Replace `Issue`, `PullRequest`, `Doc` structs with a single generic struct:

````rust
pub struct Item {
    pub id: String,                                    // UUID or slug
    pub item_type: ItemType,
    pub display_number: Option<u32>,
    pub title: String,
    pub body: String,
    pub status: Option<String>,
    pub priority: Option<u32>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub fields: HashMap<String, FieldValue>,           // type-specific fields
    pub is_org_item: bool,
    pub org_slug: Option<String>,
    pub org_display_number: Option<u32>,
}

pub enum FieldValue {
    String(String),
    Number(i64),
    Bool(bool),
    List(Vec<String>),
    Null,
}
````

### 5.3 Generic CRUD

Replace the per-type CRUD modules with a single generic engine:

````rust
pub async fn create_item(
    project_path: &Path,
    item_type: &ItemType,
    config: &CentyConfig,
    options: CreateItemOptions,
) -> Result<CreateItemResult, ItemError> {
    let schema = config.item_types.get(item_type.as_str()).unwrap();
    validate_fields(&options.fields, &schema.fields)?;

    if schema.features.status {
        validate_status(&options.status, &schema.allowed_states)?;
    }
    if schema.features.priority {
        validate_priority(options.priority, schema.priority_levels)?;
    }

    let id = match schema.identifier {
        IdentifierMode::Uuid => Uuid::new_v4().to_string(),
        IdentifierMode::Slug => slugify(&options.title),
    };

    let display_number = if schema.features.display_number {
        Some(get_next_display_number(project_path, &schema.folder).await?)
    } else {
        None
    };
    // Generate frontmatter and write file...
}
````

### 5.4 Remove Entity Modules

The following modules are replaced by the generic engine:

* `src/item/entities/issue/` (crud, metadata, frontmatter, status, priority, planning, assets, reconcile, org_registry)
* `src/item/entities/pr/` (crud, metadata, git, remote, status)
* `src/item/entities/doc/` (crud)

What remains:

* `src/item/core/` – adapted to work with the generic `Item` struct
* `src/item/operations/` – `move` and `duplicate` become generic (gated by feature flags)
* `src/item/organization/` – org sync becomes generic (gated by feature flags)
* `src/item/validation/` – field validation driven by schema

Special behaviors that were hardcoded in entity modules:

* **PR git integration** (branch detection, remote parsing): moves to hooks or a `behaviors` system (see section 7).
* **Issue assets**: generalized to any item type with `features.assets: true`.
* **Doc slugification**: handled by `identifier: "slug"` mode in the generic engine.

## 6. Links

### 6.1 Replace `LinkTargetType` Enum

Current proto:

````protobuf
enum LinkTargetType {
  LINK_TARGET_TYPE_UNSPECIFIED = 0;
  LINK_TARGET_TYPE_ISSUE = 1;
  LINK_TARGET_TYPE_DOC = 2;
  LINK_TARGET_TYPE_PR = 3;
}
````

Becomes string-based:

````protobuf
message CreateLinkRequest {
  string project_path = 1;
  string source_id = 2;
  string source_type = 3;        // "issue", "pr", "epic", etc.
  string target_id = 4;
  string target_type = 5;        // "issue", "pr", "epic", etc.
  string link_type = 6;
}
````

The daemon validates that `source_type` and `target_type` exist in `config.itemTypes`.

### 6.2 Entity Actions

The `EntityType` enum in `GetEntityActionsRequest` also becomes a string `item_type` field. Actions are derived from the item type’s feature flags.

## 7. Behavioral Hooks

Some item types have behaviors beyond data storage. Currently these are hardcoded (PR branch detection, issue planning status). In v2, these become hook-driven.

### 7.1 Hook Patterns

The existing hook pattern system already uses string-based item type names. This naturally extends to user-defined types:

````json
{
  "hooks": [
    {
      "pattern": "post:pr:create",
      "command": "git branch --show-current",
      "enabled": true
    },
    {
      "pattern": "pre:epic:update",
      "command": "./scripts/validate-epic.sh",
      "enabled": true
    }
  ]
}
````

### 7.2 Built-in Behaviors

For PR-specific git integration that’s too complex for shell hooks, introduce optional `behaviors` in the item type schema:

````json
{
  "pr": {
    "behaviors": ["git-branch-detection"],
    "fields": {
      "source_branch": { "type": "string", "required": true },
      "target_branch": { "type": "string", "required": true }
    }
  }
}
````

Available built-in behaviors:

* `git-branch-detection` – auto-populates `source_branch` from current git branch on create.

## 8. Search

### 8.1 Cross-Type Search

The `AdvancedSearch` RPC gains an `item_types` filter:

````protobuf
message AdvancedSearchRequest {
  string query = 1;
  repeated string item_types = 2;    // empty = search all types
  string sort_by = 3;
  bool sort_descending = 4;
  bool multi_project = 5;
  string project_path = 6;
}
````

### 8.2 Field-Aware Search

The search query parser accepts field names from any item type’s schema:

````
status:open AND priority:1                     # core fields
source_branch:feature/* AND item_type:pr       # type-specific field
target_date:>2025-06-01 AND item_type:epic     # date comparison
````

## 9. Templates

Templates are already organized per type. This extends naturally:

````
templates/
  issue/
    default.md
    bug.md
  pr/
    default.md
  epic/
    default.md
    quarterly.md
````

## 10. Reconciliation and Init

### 10.1 Init

`centy init` creates folder structure based on `config.json`:

* Reads `itemTypes` from config (or uses defaults if no config exists).
* Creates `.centy/{folder}/` for each defined item type.
* Creates `templates/{type}/default.md` for each type.

### 10.2 Default Item Types

When no `itemTypes` config exists (fresh init), the daemon provides default item types equivalent to the current v1 behavior: `issue`, `pr`, and `doc` with their current schemas.

## 11. Migration Path (v1 to v2)

### 11.1 Config Migration

On first read of a v1 `config.json` (detected by absence of `itemTypes` key), the daemon auto-migrates:

1. Read existing top-level fields (`priorityLevels`, `allowedStates`, `defaultState`, `stateColors`, `priorityColors`, `customFields`).
1. Generate `itemTypes.issue` using those values + default issue features.
1. Generate `itemTypes.pr` with default PR schema.
1. Generate `itemTypes.doc` with default doc schema.
1. Remove the old top-level fields.
1. Write the migrated config back to disk.

### 11.2 File Migration

No file migration needed. Existing `.centy/issues/`, `.centy/prs/`, `.centy/docs/` directories and their markdown files remain valid because:

* Folder names in the default item type configs match existing folders.
* Frontmatter fields are a superset – the generic parser handles both old and new formats.
* UUID and slug identification modes match existing behavior.

### 11.3 gRPC Migration

With Option A (parallel v1/v2 packages):

1. v2 RPCs are the primary implementation.
1. v1 RPCs are thin wrappers that call v2 with hardcoded `item_type` values.
1. v1 is deprecated and removed in a future version.

### 11.4 Manifest Version Bump

The manifest `schema_version` bumps from 1 to 2 to signal that this project has been migrated.

## 12. Asset Generalization

Currently assets are issue-only. In v2, assets attach to any item type with `features.assets: true`:

````protobuf
message AddAssetRequest {
  string project_path = 1;
  string item_type = 2;
  string item_id = 3;
  string filename = 4;
  bytes data = 5;
  bool is_shared = 6;
}
````

Storage: `.centy/assets/{item_type}/{item_id}/{filename}`.

## 13. Workspace Generalization

Currently workspace RPCs are tied to issues. In v2, workspaces work with any item type:

````protobuf
message OpenInTempWorkspaceWithEditorRequest {
  string project_path = 1;
  string item_type = 2;
  string item_id = 3;
  LlmAction action = 4;
  string agent_name = 5;
  uint32 ttl_hours = 6;
  string editor_id = 7;
}
````

## 14. ProjectInfo Update

Current `ProjectInfo` has hardcoded counts (`issue_count`, `doc_count`). Replace with:

````protobuf
map<string, uint32> item_counts = 4;  // e.g. {"issue": 42, "pr": 7, "epic": 3}
````

## 15. Implementation Phases

### Phase 1: Config Schema

* Define `ItemTypeSchema` struct in Rust.
* Add `item_types` field to `CentyConfig`.
* Implement v1-to-v2 config auto-migration.
* Write config validation (unique folder names, valid field types, etc.).

### Phase 2: Generic Item Engine

* Create generic `Item` struct and `FieldValue` enum.
* Implement generic frontmatter parser/writer driven by schema.
* Implement generic CRUD (create, get, list, update, delete, soft-delete, restore).
* Implement display number management per item type.
* Implement field validation against schema.

### Phase 3: Proto v2

* Define `centy.v2` proto package with generic Item RPCs.
* Implement v2 gRPC handlers using the generic engine.
* Add v1 compatibility shim.

### Phase 4: Feature-Gated Operations

* Generalize move/duplicate (gated by feature flags).
* Generalize assets (gated by feature flags).
* Generalize org-sync (gated by feature flags).

### Phase 5: Search and Links

* Update advanced search to accept `item_types` filter.
* Update search parser to handle schema-defined fields.
* Replace `LinkTargetType` enum with string-based types.
* Update entity actions to be schema-driven.

### Phase 6: Templates and Reconciliation

* Update template engine for generic item types.
* Update init/reconciliation for dynamic folder creation.
* Update workspace RPCs to work with any item type.

### Phase 7: Cleanup

* Remove old entity modules (`issue/`, `pr/`, `doc/` under `src/item/entities/`).
* Remove v1 proto package (after deprecation period).
* Remove `ItemType` enum.
* Update all tests.

## 16. Open Questions

1. **PR git behaviors**: Should `git-branch-detection` be a built-in behavior, a hook, or a daemon plugin system?

1. **Field indexing for search**: With arbitrary fields, should the daemon build field indexes for efficient search, or continue with full-scan + in-memory filtering?

1. **Cross-type display numbers**: Should display numbers be global (across all types) or per-type? Per-type matches current behavior.

1. **Config validation timing**: Should invalid item type configs prevent daemon startup, or just log warnings and skip?

1. **Workspace RPCs**: Currently tied to issues. Should these work with any item type?

1. **Per-type assets folder**: Should assets live at `.centy/assets/{item_type}/{item_id}/` or remain flat at `.centy/assets/{item_id}/`?

1. **Config service split**: The recently extracted `ConfigService` (issue #158) – should item type management get its own gRPC service, or stay within config?
