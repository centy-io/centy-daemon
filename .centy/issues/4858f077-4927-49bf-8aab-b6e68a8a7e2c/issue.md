# Issue Conversation Feature

Enable team discussions and decision logging on issues (and other entities).

---

## Design Decisions

| Area | Decision |
|------|----------|
| **Use Cases** | Async team discussions + Decision logging |
| **Storage** | File-based, git-friendly |
| **Threading** | Flat + threaded replies using CRDT |
| **CRDT Library** | Yrs (Yjs Rust port) |
| **File Format** | JSON (human-readable, git-diff friendly) |
| **File Location** | Entity folder (`.centy/{entity-type}/{id}/conversation.json`) |
| **Edit/Delete** | Tombstones only (mark deleted, keep last edit) |
| **API** | Generic `ConversationService` (entity-agnostic) |
| **Scope** | Start with issues, design for all entities |
| **Schema** | Minimal v1, extend later |

---

## Why CRDT?

CRDTs (Conflict-free Replicated Data Types) solve the git merge conflict problem:

- Multiple contributors can add comments offline
- Git merges become automatic (no conflicts)
- Order is preserved even with concurrent edits
- Deletions/edits represented as operations

---

## Minimal Comment Schema (v1)

```
Comment {
  id: UUID
  author: { name, email }  // from git config
  content: String (Markdown)
  parent_id: Option<UUID>  // for threading (null = top-level)
  created_at: Timestamp
  updated_at: Option<Timestamp>
  deleted: bool  // tombstone marker
}
```

---

## File Structure

```
.centy/issues/{uuid}/
├── issue.md          # Title + Description
├── metadata.json     # Status, priority, timestamps
├── links.json        # Related issues/docs/PRs
├── conversation.json # NEW: CRDT-backed conversation
└── assets/           # Attached files
```

---

## API Design

Generic `ConversationService` that works with any entity type:

- `AddComment(entity_type, entity_id, content, parent_id?)`
- `ListComments(entity_type, entity_id)`
- `UpdateComment(entity_type, entity_id, comment_id, content)`
- `DeleteComment(entity_type, entity_id, comment_id)` (tombstone)

---

## Open Questions (for future)

1. **Git Merge Strategy**: Does Yrs handle JSON merging automatically? Need custom git merge driver?

2. **CLI Commands**: How should `centy` CLI expose conversations?
   - `centy issue comment add <id> "message"`?
   - `centy issue conversation <id>`?

3. **Notifications/Mentions**: Out of scope for v1

4. **Performance**: How does Yrs JSON export scale with large conversations?

5. **Real-time sync**: Future consideration for collaborative editing
