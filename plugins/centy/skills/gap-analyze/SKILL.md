---
name: gap-analyze
description: This skill should be used when the user asks to "analyze gaps in an epic", "check user story coverage", "gap analyze epic", "make sure an epic has user stories", "validate stories for an epic", or wants to ensure an epic's goals are fully covered by well-formed user stories.
allowed-tools: mcp__centy__IsRunning mcp__centy__StartDaemon mcp__centy__centy_v1_CentyDaemon_IsInitialized mcp__centy__centy_v1_CentyDaemon_ListItems mcp__centy__centy_v1_CentyDaemon_GetItem mcp__centy__centy_v1_CentyDaemon_ListLinks mcp__centy__centy_v1_CentyDaemon_ListItemTypes mcp__centy__centy_v1_CentyDaemon_CreateItem mcp__centy__centy_v1_CentyDaemon_UpdateItem mcp__centy__centy_v1_CentyDaemon_CreateLink
---

# gap-analyze

Inspect an epic's stated goals, find any areas not covered by a well-formed user story, create or update stories to fill those gaps, then flag any story that still fails the quality bar by raising an issue.

When called with an epic ID or display number, operate on that epic. When called with no arguments, auto-discover all open or in-progress epics in the project and process each one in turn.

---

## Passing criteria for a user story

A story **passes** if it satisfies ALL of the following:

1. **Body present** — the body is non-empty and contains more than a one-liner.
2. **User-story statement** — the body includes a statement of the form *"As a …, I want …, so that …"* (case-insensitive).
3. **Acceptance criteria** — the body has a heading that contains "acceptance criteria" (or "AC") followed by at least one checklist item or bullet point.
4. **Status is not `draft`** — the story has been promoted to `"ready"` or `"done"`.

A story **fails** if it violates ANY of the above.

---

## Workflow

### 0 — Prerequisites

Ensure the daemon is running and the project is initialized. Resolve `project_path` from the current working directory. Start the daemon with `StartDaemon` if `IsRunning` returns false.

---

### 1–3 — Research phase (run in a sub-agent)

Launch a **read-only sub-agent**. It must not create, update, or delete anything. Hand it:
- `epic_id` or `epic_display_number` if one was provided; otherwise `"discover"`
- `project_path`

**Sub-agent prompt template:**

```
You are performing the research phase of a "gap-analyze" workflow.
Do NOT create, update, or delete any items — read only.

project_path: <project_path>
Epic to analyze: <epic_id | epic_display_number | "discover">

Steps:

0. RESOLVE THE EPIC(S)
   If an ID or display number was given, fetch it with GetItem (item_type: "epics").
   If "discover" was given, call ListItems (item_type: "epics") and select all epics
   whose status is "open" or "in-progress". Return one REPORT block per epic.
   If no open/in-progress epics exist, select ALL epics regardless of status.
   Process each epic independently through steps 1–4 below.

1. READ THE EPIC FULLY
   Capture:
   - id, display_number, title, body, status, tags, custom_fields

2. EXTRACT INTENDED SCOPE FROM THE EPIC BODY
   Parse the epic body for any of these signals (each becomes a "scope item"):
   - Numbered or bulleted lists describing features, goals, or capabilities
   - Headings (##, ###) naming areas of functionality
   - Sentences starting with "The system should…", "Users should be able to…",
     "Support…", "Enable…", "Allow…", "As a…"
   - Acceptance criteria blocks already written into the epic
   If the epic body is empty or too vague to extract scope, record a single
   scope item: "(no structured scope found — generic story needed)".

3. DISCOVER LINKED USER STORIES
   a. Call ListLinks for the epic ID.
   b. For each link where link_type is "parent-of" and target_item_type is "story":
      - Fetch the story with GetItem (item_type: "stories").
      - Record: id, display_number (if any), title, body, status.
      - Classify the story:
          PASS  — meets all four passing criteria (body present, user-story
                  statement, acceptance criteria section, status != "draft")
          FAIL  — violates one or more criteria; list which ones

4. MAP COVERAGE
   For each scope item, decide if an existing (linked) story covers it:
   - COVERED   — at least one linked story's title or body clearly addresses
                 this scope item
   - UNCOVERED — no linked story addresses this scope item

Return a single REPORT block:

REPORT:
  epic_id: <UUID>
  epic_display_number: <N>
  epic_title: <string>
  scope_items:
    - text: <string>
      covered: <true | false>
      covering_story_ids: [<UUID>, …]   # empty if uncovered
  existing_stories:
    - id: <UUID>
      display_number: <N | null>
      title: <string>
      status: <string>
      result: <"pass" | "fail">
      fail_reasons: [<string>, …]       # empty if pass
  warnings: [<string>, …]
```

Wait for all REPORT blocks before continuing. If the sub-agent returned multiple REPORTs (discover mode), process them one at a time: complete steps 4–8 for the first epic, then move on to the next — no user input between epics.

---

### 4 — Plan story changes

Using the REPORT, build an action list:

**For each UNCOVERED scope item:**
- Check if any existing story (even one that failed) partially covers it.
  - If yes → mark that story for **UPDATE** with the uncovered scope item merged in.
  - If no  → plan to **CREATE** a new story for this scope item.

**For each existing story with result `fail`:**
- If it is already marked for UPDATE (from the step above) → the update will also fix the failing fields.
- If it is not yet in the update list → plan to **UPDATE** it to meet the passing criteria.

---

### 5 — Create new stories

For each planned CREATE:

**Story body template:**

```markdown
## User story

As a [inferred persona from epic context], I want [capability derived from scope item], so that [benefit derived from epic goals].

## Acceptance criteria

- [ ] [Criterion 1 — derived from the scope item and epic body]
- [ ] [Criterion 2]
- [ ] [Add more as needed]

## Notes

Scope item: <verbatim scope item text>
Epic: #<epic_display_number> <epic_title>
```

Steps:
1. `CreateItem` with `item_type: "stories"`, `title`, `body` (filled from template), `status: "ready"`.
2. `CreateLink` from the epic to the new story:
   - `source_id`: epic UUID
   - `target_id`: new story UUID
   - `link_type`: `"parent-of"`
   - `source_item_type`: `"epic"`
   - `target_item_type`: `"story"`
3. Verify the story passes all four criteria. If it does, mark it `result: "pass"`.

---

### 6 — Update existing stories

For each planned UPDATE:

Merge in any missing content:
- If missing the user-story statement → prepend it above the existing body.
- If missing acceptance criteria → append an `## Acceptance criteria` section with generated criteria based on the scope item and epic context.
- If status is `"draft"` → set `status: "ready"`.

Steps:
1. `UpdateItem` with `item_type: "stories"`, the story UUID, and the revised `body` and/or `status`.
2. Re-evaluate the story against the four passing criteria.
3. If the story now passes, mark `result: "pass"`. If it still fails (e.g., body content was too ambiguous to generate good ACs), mark `result: "fail"` and record the remaining fail reasons.

---

### 7 — Create issues for stories that still fail

For each story that still has `result: "fail"` after steps 5–6:

Create one issue per failing story using the template:

**Issue title:**
`User story #<display_number> "<title>" does not meet quality bar`

**Issue body template:**

```markdown
## Problem

User story #<display_number> linked to epic #<epic_display_number> **<epic_title>** does not meet the story quality bar.

## Failing criteria

<for each fail_reason, a bullet like:>
- Missing user-story statement ("As a … I want … so that …")
- Missing acceptance criteria section
- Status is still "draft"
- Body is empty or nearly empty

## What needs to be done

Review and update the story so it satisfies all failing criteria above.

## Links

- Story: #<display_number> <title>
- Parent epic: #<epic_display_number> <epic_title>
```

Steps:
1. `CreateItem` with `item_type: "issues"`, `title`, `body`, `status: "open"`, `priority: 2`.
2. `CreateLink` from the new issue to the story:
   - `source_id`: issue UUID
   - `target_id`: story UUID
   - `link_type`: `"blocks"`
   - `source_item_type`: `"issue"`
   - `target_item_type`: `"story"`

---

### 8 — Report to the user

Print a summary using this format:

```
Gap analysis: Epic #<N> — <title>

Scope coverage
  Covered (<X> items): ✓ <scope item text> → #<story_N> <story_title>
                        …
  Uncovered (<Y> items): ✗ <scope item text>
                          …

Stories
  Created  (<A>): #<N> <title>
  Updated  (<B>): #<N> <title>
  Passed   (<C>): #<N> <title>  [already passing — no changes needed]
  Failed   (<D>): #<N> <title>  → issue #<M> raised

Issues raised
  #<M> <issue_title>
  …
```

Surface any `warnings` from the sub-agent. If no gaps were found and all stories already pass, report that the epic is fully covered and no changes were made.

---

## Edge cases

| Situation | Handling |
|-----------|----------|
| No epic ID given | Auto-discover all open/in-progress epics; process each one |
| Epic body is empty | Generate one generic story covering the epic title; note the lack of scope detail in the issue if the story can't meet the bar |
| Epic has no linked stories at all | Create one story per extracted scope item (or one generic story if scope is empty) |
| All stories already pass | Report "fully covered — no changes needed"; skip steps 5–7 |
| A story passes after update | Do not raise an issue for it |
| A scope item maps to multiple stories | Mark as COVERED; no new story needed |
| Story type not available in project | Fall back to `item_type: "issues"` and note the fallback in the report |
| Epic not found | Report the error and stop |
| Link creation fails | Log the failure as a warning and continue; do not abort the whole run |
