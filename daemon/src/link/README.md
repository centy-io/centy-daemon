# Links

The link system lets you create typed, bidirectional relationships between entities (items, docs, etc.). Links are stored as individual markdown files in `.centy/links/`.

## Link types

Built-in link types (always available):

| Link type | Inverse |
|-----------|---------|
| `blocks` | `blocked-by` |
| `parent-of` | `child-of` |
| `relates-to` | `related-from` |
| `duplicates` | `duplicated-by` |

Custom link types can be defined in `config.json`:

```json
{
  "linkTypes": [
    { "name": "implements", "description": "Implements a spec or requirement" },
    { "name": "tested-by" }
  ]
}
```

## Storage

Each link is stored as one markdown file in `.centy/links/<uuid>.md` with YAML front matter:

```
---
id: <uuid>
source_id: <uuid>
source_type: issue
target_id: <uuid>
target_type: issue
link_type: blocks
created_at: 2024-01-15T10:30:00Z
updated_at: 2024-01-15T10:30:00Z
---
```

Links are bidirectional — one file represents the relationship from both sides. When queried from the source's perspective the link type is returned as-is (e.g. `blocks`). When queried from the target's perspective the same link type is returned, and `direction` is set to `target` so the caller knows which side it is on.

## API

| Function | Description |
|----------|-------------|
| `create_link(project_path, opts)` | Create a new link between two entities |
| `delete_link(project_path, opts)` | Delete a link by ID |
| `delete_link_by_id(project_path, id)` | Delete a link by UUID directly |
| `list_links(project_path, entity_id, entity_type)` | List all links for an entity (both directions) |
| `get_available_link_types(custom_types)` | Return all built-in and custom link types |

### CreateLinkOptions

| Field | Type | Description |
|-------|------|-------------|
| `source_id` | String | UUID of the source entity |
| `source_type` | TargetType | Type of the source entity (e.g. `issue`) |
| `target_id` | String | UUID of the target entity |
| `target_type` | TargetType | Type of the target entity |
| `link_type` | String | Must be a built-in or configured custom link type |

### Errors

| Error | Description |
|-------|-------------|
| `InvalidLinkType` | The link type is not built-in and not in custom types |
| `SourceNotFound` | Source entity does not exist |
| `TargetNotFound` | Target entity does not exist |
| `LinkAlreadyExists` | An identical link already exists |
| `LinkNotFound` | Link UUID does not exist (on delete) |
| `SelfLink` | Source and target are the same entity |
