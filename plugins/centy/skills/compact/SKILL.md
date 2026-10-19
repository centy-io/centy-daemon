---
name: compact
description: This skill should be used when the user asks to "compact issues", "consolidate issues", "merge issues into an epic", "roll up related issues", or wants to group completed issues under a feature item and soft-delete the ones that are done.
allowed-tools: mcp__centy__centy_v1_CentyDaemon_IsInitialized mcp__centy__centy_v1_CentyDaemon_ListItems mcp__centy__centy_v1_CentyDaemon_GetItem mcp__centy__centy_v1_CentyDaemon_ListLinks mcp__centy__centy_v1_CentyDaemon_ListItemTypes mcp__centy__centy_v1_CentyDaemon_CreateItem mcp__centy__centy_v1_CentyDaemon_UpdateItem mcp__centy__centy_v1_CentyDaemon_CreateLink mcp__centy__centy_v1_CentyDaemon_SoftDeleteItem
---

# compact

Consolidate a set of related issues that share a feature domain into a single feature item (epic), then soft-delete the issues that have already been completed. Use this skill whenever the user asks to compact, consolidate, merge, or roll up a group of related issues.

When called with no arguments, the skill discovers which issues to compact on its own — it never asks the user to provide a list.

## What this skill does

1. Discovers issues to compact (from user input, or automatically from the project)
2. Synthesizes the underlying feature from their content
3. Finds an existing epic for that feature — or creates one
4. Creates/updates the epic body with a full implementation summary
5. Links still-active issues as children of the epic
6. Soft-deletes issues whose work is already done

---

## Workflow

### 0 — Prerequisites

Ensure the daemon is running and the project is initialized. Resolve `projectPath` from the current working directory or ask the user if ambiguous.

---

### 1–4 — Research phase (run in an Agent)

Launch a **sub-agent** whose sole job is to research and synthesize — it must not write anything. Hand it:
- The list of issue IDs or display numbers **if the user provided any**, otherwise an empty list meaning "discover everything"
- `projectPath`

The sub-agent must be read-only. It must not create, update or delete any item. Its steps:

0. **Discover issues** (only when no issues were given). List all issues of type `"issues"` in the project. Prefer issues whose status is closed, done or resolved, since they are the prime candidates for roll-up. Also include open or in-progress issues that clearly belong to the same feature domain. Exclude issues that already have a parent link to a feature item. Group the candidates into one or more feature clusters by subject matter; each cluster becomes one report. If no candidates exist, return one report with an empty issue list and a warning that says why.
1. **Read every issue.** Fetch each issue's full content and existing links. If a display number was given instead of an ID, resolve it to an ID first. Classify each issue as done (status closed, done or resolved) or active (any other non-terminal status).
2. **Infer the feature domain.** From the collected issues, synthesize a feature name (80 characters or fewer), a 2–4 sentence summary, the done work, the pending work, cross-cutting implementation notes, the deduplicated union of all tags, and the highest priority value seen.
3. **Discover the feature item type.** List the item types for the project. Pick the first available type from: epics, features, stories, issues. The issues type is always available as a fallback.
4. **Check for an existing feature item.** Search for a feature item whose title matches the feature name's key terms. Follow any child or parent link from step 1 to find a candidate and read it. If a candidate's domain clearly matches, record its ID and title. If a candidate exists but its domain clearly differs, record no match and add a warning about the mismatch. If no candidate exists, record no match.

Return one report per feature cluster. Each report contains every field below, with none omitted:
- feature type, feature name, feature summary
- done items list, pending items list, implementation notes
- tags list, highest priority
- existing feature ID and title (or none)
- the issue list, where each entry has ID, display number, title, classification (done or active), and whether it is already linked to the feature
- warnings list (not-found issues, domain mismatches and similar)

Wait for the agent to return its report(s) before continuing.

---

### 5–8 — Execute one cluster at a time

If the sub-agent returned multiple reports, process them one at a time: complete steps 5–8 for the first cluster, report the result to the user, then move on to the next cluster. Do not start a new cluster until the previous one is fully done.

### 5 — Create or update the feature item

Use the report to build the feature body using the template below, then:
- If an existing feature ID is set — update that item with the synthesized content, and refine the title only if it clearly improves on the existing one.
- If no existing feature ID is set — create a new item of the reported feature type.

Set status to `"in-progress"` if any active issues remain, or `"closed"` if all issues are done.

**Feature body template:**

```markdown
## Overview

<feature_summary>

## Done

- <done_items…>

## Pending

- <pending_items…>

## Implementation notes

<implementation_notes>
```

Omit `## Done` if the done list is empty. Omit `## Pending` if the pending list is empty.

---

### 6 — Link active issues to the feature item

For each active issue that isn't already linked to the feature item, create a `"child"` link from the feature item to the issue.

---

### 7 — Soft-delete completed issues

For each done issue, soft-delete it without a forced hard delete. This sets the deleted-at marker without removing the file — the issue is hidden from normal queries but remains recoverable. Never soft-delete active issues.

---

### 8 — Report to the user

```
Feature item: #<displayNumber> — <title> [created | updated]
Linked (active): #N1 <title>, #N2 <title>, …
Soft-deleted (done): #N3 <title>, #N4 <title>, …
```

Surface any warnings the agent returned. If more clusters remain, proceed to the next one after reporting this result.

---

## Edge cases

| Situation | Handling |
|-----------|----------|
| No input given | Auto-discover all issues eligible for compact in the project (step 0) |
| No eligible issues found | Report to user and stop — nothing to do |
| All issues are already closed | Feature status → `"closed"`; soft-delete all |
| All issues are open | Feature status → `"in-progress"`; link all, delete none |
| An issue is not found | Agent records a warning; skip it, continue with the rest |
| Issue already linked to a different feature item | Note the conflict; link to the new feature anyway unless user says otherwise |
| `feature_type` is null | Cannot happen — "issues" is always the final fallback |
| Multiple distinct feature clusters discovered | Process one cluster at a time — finish and report before starting the next |
| User provides mixed project paths | Group issues by project; run the full workflow per project |
