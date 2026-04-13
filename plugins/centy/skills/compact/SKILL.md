---
name: compact
description: This skill should be used when the user asks to "compact issues", "consolidate issues", "merge issues into an epic", "roll up related issues", or wants to group completed issues under a feature item and soft-delete the ones that are done.
version: 1.0.0
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

Ensure the daemon is running and the project is initialized. Resolve `project_path` from the current working directory or ask the user if ambiguous.

---

### 1–4 — Research phase (run in an Agent)

Launch a **sub-agent** whose sole job is to research and synthesize — it must not write anything. Hand it:
- The list of issue IDs or display numbers **if the user provided any**, otherwise an empty list meaning "discover everything"
- `project_path`

**Sub-agent prompt template:**

```
You are performing the research phase of a "compact" workflow.
Do NOT create, update, or delete any items — read only.

project_path: <project_path>
Issues to research: <id or display number list, or "discover" if none given>

Steps:

0. DISCOVER ISSUES (only when "Issues to research" is "discover")
   List ALL issues in the project (item_type: "issues").
   From that full list, select candidates for compacting:
   - Prefer issues that are "closed", "done", or "resolved" — these are prime
     candidates to be rolled up and soft-deleted.
   - Also include any open/in-progress issues that clearly belong to the same
     feature domain as the closed ones.
   - Exclude issues that already have a parent link to an existing feature item
     (they were already compacted).
   - Group the candidates into one or more feature clusters based on subject
     matter. Each cluster will become one report. If there are multiple distinct
     clusters, return a separate REPORT block for each.
   - If no candidates are found, return a single report with
     issues: [] and a warning explaining why.

1. READ EVERY ISSUE
   Fetch the full content and existing links for each issue in the cluster.
   If a display number was given instead of a UUID, resolve it to a UUID first.
   Classify each issue:
   - Done   → status is "closed", "done", or "resolved"
   - Active → status is "open", "in-progress", or any other non-terminal status

2. INFER THE FEATURE DOMAIN
   From the collected issues, synthesize:
   - feature_name        Short, precise title (≤ 80 chars)
   - feature_summary     2–4 sentences: what the feature does, why it exists
   - done_items          Bullet list of completed work (from Done issues)
   - pending_items       Bullet list of remaining work (from Active issues)
   - implementation_notes  Cross-cutting design decisions, constraints, gotchas
   - tags                Deduplicated union of all tags across every issue
   - highest_priority    Numerically highest priority value seen across all issues

3. DISCOVER THE FEATURE ITEM TYPE
   List the available item types for this project.
   Pick the first available type from: epics → features → stories → issues.
   feature_type will always be set — there is always an "issues" fallback.

4. CHECK FOR AN EXISTING FEATURE ITEM
   a. Search for a feature item whose title matches key terms from feature_name.
   b. Inspect the links from step 1 — follow any "child" or "parent" link to find
      a candidate feature item and read its content.
   Decision:
   - If a candidate's domain clearly matches → set existing_feature_id to its ID
     and existing_feature_title to its title
   - If a candidate exists but domain clearly differs → set existing_feature_id
     to null and note the mismatch as a warning
   - If no candidate found → set existing_feature_id to null

Return one REPORT block per feature cluster. Each block must contain ALL of the
following fields — no fields omitted:

REPORT:
  feature_type: <"epics" | "features" | "stories" | "issues">
  feature_name: <string>
  feature_summary: <string>
  done_items: [<string>, …]
  pending_items: [<string>, …]
  implementation_notes: <string>
  tags: [<string>, …]
  highest_priority: <int>
  existing_feature_id: <UUID | null>
  existing_feature_title: <string | null>
  issues:
    - id: <UUID>
      display_number: <N>
      title: <string>
      classification: <"done" | "active">
      already_linked_to_feature: <bool>
  warnings: [<string>, …]   # not-found issues, domain mismatches, etc.
```

Wait for the agent to return its report(s) before continuing.

---

### 5–8 — Execute for each report

Run steps 5–8 independently for every REPORT block the sub-agent returned.

### 5 — Create or update the feature item

Use the report to build the feature body using the template below, then:
- If `existing_feature_id` is set — update that item with the synthesized content, and refine the title only if it clearly improves on the existing one.
- If `existing_feature_id` is null — create a new item of `feature_type`.

Set status to `"in-progress"` if any Active issues remain, or `"closed"` if all issues are Done.

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

Omit `## Done` if `done_items` is empty. Omit `## Pending` if `pending_items` is empty.

---

### 6 — Link active issues to the feature item

For each Active issue that isn't already linked to the feature item, create a `"child"` link from the feature item to the issue.

---

### 7 — Soft-delete completed issues

For each Done issue, soft-delete it (`force: false`). This sets `deleted_at` without removing the file — the issue is hidden from normal queries but remains recoverable. Never soft-delete Active issues.

---

### 8 — Report to the user

```
Feature item: #<display_number> — <title> [created | updated]
Linked (active): #N1 <title>, #N2 <title>, …
Soft-deleted (done): #N3 <title>, #N4 <title>, …
```

If multiple clusters were processed, print one block per cluster. Surface any warnings the agent returned.

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
| Multiple distinct feature clusters discovered | Run steps 5–8 once per cluster |
| User provides mixed project paths | Group issues by project; run the full workflow per project |
