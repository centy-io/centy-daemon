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

**a. Check for MCP tools.**
Try to call `IsRunning`. If the `mcp__centy__*` tools are not available (not in the tool list / return a tool-not-found error), fall back to the CLI for all operations: use `pnpm dlx centy` for every read and write action throughout this workflow. Note the degraded mode in the final report. Do not abort — continue with the CLI fallback.

**b. Check version compatibility.**
After confirming the daemon is reachable, run `pnpm dlx centy config` (or `GetDaemonInfo` via MCP) and compare the project version against the daemon version. If they differ, stop and tell the user:

> Version mismatch detected: project is at X, daemon is at Y.
> Run `pnpm dlx centy init` to migrate, then re-run gap-analyze.

Do not proceed past step 0 until versions match.

**c. Start the daemon if needed.**
If `IsRunning` returns false, call `StartDaemon`. Resolve `project_path` from the current working directory. Confirm `IsInitialized` before continuing.

---

### 1–3 — Research phase (run in a sub-agent)

Launch a **read-only sub-agent**. It must not create, update, or delete anything. Hand it:
- `epic_id` or `epic_display_number` if one was provided; otherwise `"discover"`
- `project_path`
- `mcp_available`: true or false (from step 0a above)

**Sub-agent prompt template:**

```
You are performing the research phase of a "gap-analyze" workflow.
Do NOT create, update, or delete any items — read only.

project_path: <project_path>
Epic to analyze: <epic_id | epic_display_number | "discover">
MCP available: <true | false>

If MCP is not available, use `pnpm dlx centy` CLI commands for all reads.

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
   If the epic body contains explicit "Done" / "Pending" (or "Complete" / "Remaining",
   or "## Done" / "## Pending") sections, extract scope items ONLY from the Pending
   section — items in Done are already shipped and require no story coverage.

   Otherwise, parse the full body for any of these signals (each becomes a "scope item"):
   - Numbered or bulleted lists describing features, goals, or capabilities
   - Headings (##, ###) naming areas of functionality
   - Sentences starting with "The system should…", "Users should be able to…",
     "Support…", "Enable…", "Allow…", "As a…"
   - Acceptance criteria blocks already written into the epic

   If there are more than 5 pending scope items, group related items by theme into
   at most 3 thematic groups. Each group becomes one scope item in the report, with
   the constituent items listed under it. This keeps story count manageable.

   If the epic body is empty or too vague to extract scope, record a single
   scope item: "(no structured scope found — generic story needed)".

3. DISCOVER LINKED USER STORIES
   a. Call ListLinks for the epic ID (or `pnpm dlx centy list links --item epic:<id>`).
   b. For each link returned, read the `linkType` AND `targetType` (or `sourceType`)
      fields verbatim from the link record. Only proceed with items where:
        - linkType  = "parent-of"  AND  targetType = "story"   (epic → story), OR
        - linkType  = "child-of"   AND  sourceType = "story"   (story → epic)
      Do NOT classify items of other targetTypes (e.g. "issue", "doc") as stories,
      even if they appear in a parent-of link.
   c. For each qualifying story link, fetch the story with GetItem (item_type: "stories")
      using the story's identifier (UUID or slug, whichever the targetId contains).
      Record: id, title, body, status (empty status = "draft").
   d. Classify each story:
          PASS  — meets all four passing criteria (body present, user-story
                  statement, acceptance criteria section, status != "draft")
          FAIL  — violates one or more criteria; list which ones

4. MAP COVERAGE
   For each scope item (or thematic group), decide if an existing linked story covers it:
   - COVERED   — at least one linked story's title or body clearly addresses
                 this scope item or group
   - UNCOVERED — no linked story addresses this scope item or group

Return a single REPORT block:

REPORT:
  epic_id: <UUID>
  epic_display_number: <N>
  epic_title: <string>
  scope_items:
    - text: <string>
      grouped_items: [<string>, …]     # constituent items if this is a thematic group
      covered: <true | false>
      covering_story_ids: [<id>, …]    # empty if uncovered
  existing_stories:
    - id: <id>                         # UUID or slug
      title: <string>
      status: <string>                 # treat empty as "draft"
      result: <"pass" | "fail">
      fail_reasons: [<string>, …]      # empty if pass
  warnings: [<string>, …]
```

Wait for all REPORT blocks before continuing. If the sub-agent returned multiple REPORTs (discover mode), process them one at a time: complete steps 4–8 for the first epic, then move on to the next — no user input between epics.

---

### 4 — Plan story changes

Using the REPORT, build an action list:

**For each UNCOVERED scope item (or thematic group):**
- Check if any existing story (even one that failed) partially covers it.
  - If yes → mark that story for **UPDATE** with the uncovered scope merged in.
  - If no  → plan to **CREATE** one new story covering the entire scope item or group.

**For each existing story with result `fail`:**
- If it is already marked for UPDATE (from the step above) → the update will also fix the failing fields.
- If it is not yet in the update list → plan to **UPDATE** it to meet the passing criteria.

---

### 5 — Create new stories

For each planned CREATE:

**Story body template:**

```markdown
## User story

As a [inferred persona from epic context], I want [capability derived from scope item or group], so that [benefit derived from epic goals].

## Acceptance criteria

- [ ] [Criterion 1 — derived from the scope item and epic body]
- [ ] [Criterion 2]
- [ ] [Add more as needed]

## Scope

Epic: #<epic_display_number> <epic_title>
Covers: <verbatim scope item text, or list of grouped items>
```

Steps:
1. `CreateItem` with `item_type: "stories"`, `title`, `body` (filled from template), `status: "ready"`.
   - Via CLI fallback: `pnpm dlx centy create story --title "..." --status ready --body "..."`
2. `CreateLink` from the epic to the new story:
   - `source_id`: epic UUID
   - `target_id`: new story id (UUID or slug)
   - `link_type`: `"parent-of"`
   - `source_item_type`: `"epic"`
   - `target_item_type`: `"story"`
   - Via CLI fallback: `pnpm dlx centy update epic <display_number> --link parent-of:story:<slug>`
3. **Verify the link was actually created.** Fetch the story's links and confirm the epic appears. If the link is absent, log a warning: "Link creation reported success but link was not found — story `<id>` is unlinked." Do not raise an issue for this; it is an infrastructure bug, not a story quality failure.
4. Verify the story passes all four criteria. If it does, mark it `result: "pass"`.

---

### 6 — Update existing stories

For each planned UPDATE:

Merge in any missing content:
- If missing the user-story statement → prepend it above the existing body.
- If missing acceptance criteria → append an `## Acceptance criteria` section with generated criteria based on the scope item and epic context.
- If status is `"draft"` or empty → set `status: "ready"`.

Steps:
1. `UpdateItem` with `item_type: "stories"`, the story id, and the revised `body` and/or `status`.
   - Via CLI fallback: `pnpm dlx centy update story <slug> --status ready --body "..."`
2. Re-evaluate the story against the four passing criteria.
3. If the story now passes, mark `result: "pass"`. If it still fails, mark `result: "fail"` and record the remaining fail reasons.

---

### 7 — Create issues for stories that still fail

For each story that still has `result: "fail"` after steps 5–6:

Create one issue per failing story using the template:

**Issue title:**
`User story "<title>" does not meet quality bar`

**Issue body template:**

```markdown
## Problem

User story "<title>" linked to epic #<epic_display_number> **<epic_title>** does not meet the story quality bar.

## Failing criteria

<for each fail_reason, a bullet like:>
- Missing user-story statement ("As a … I want … so that …")
- Missing acceptance criteria section
- Status is still "draft"
- Body is empty or nearly empty

## What needs to be done

Review and update the story so it satisfies all failing criteria above.

## Links

- Story: <id> <title>
- Parent epic: #<epic_display_number> <epic_title>
```

Steps:
1. `CreateItem` with `item_type: "issues"`, `title`, `body`, `status: "open"`, `priority: 2`.
2. `CreateLink` from the new issue to the story:
   - `source_id`: issue UUID
   - `target_id`: story id
   - `link_type`: `"blocks"`
   - `source_item_type`: `"issue"`
   - `target_item_type`: `"story"`

---

### 8 — Report to the user

Print a summary using this format:

```
Gap analysis: Epic #<N> — <title>

Scope coverage
  Covered (<X> items): ✓ <scope item text> → <story_id> <story_title>
                        …
  Uncovered (<Y> items): ✗ <scope item text>
                          …

Stories
  Created  (<A>): <id> <title>
  Updated  (<B>): <id> <title>
  Passed   (<C>): <id> <title>  [already passing — no changes needed]
  Failed   (<D>): <id> <title>  → issue #<M> raised

Issues raised
  #<M> <issue_title>
  …
```

Surface any `warnings` from the sub-agent and any link-verification failures from step 5. If no gaps were found and all stories already pass, report that the epic is fully covered and no changes were made.

---

## Edge cases

| Situation | Handling |
|-----------|----------|
| MCP tools unavailable | Fall back to `pnpm dlx centy` CLI; note degraded mode in report |
| Version mismatch (project vs daemon) | Stop at step 0; tell user to run `centy init` |
| No epic ID given | Auto-discover all open/in-progress epics; process each one |
| Epic body is empty | Generate one generic story covering the epic title; note the lack of scope detail |
| Epic has Done/Pending sections | Extract scope only from Pending; ignore Done items |
| More than 5 pending scope items | Group by theme into ≤ 3 thematic groups; create one story per group |
| Epic has no linked stories at all | Create one story per scope item/group (or one generic story if scope is empty) |
| All stories already pass | Report "fully covered — no changes needed"; skip steps 5–7 |
| A story passes after update | Do not raise an issue for it |
| A scope item maps to multiple stories | Mark as COVERED; no new story needed |
| Link verification fails after create/update | Log as warning; do not raise a quality issue |
| Story type not available in project | Fall back to `item_type: "issues"` and note the fallback in the report |
| Epic not found | Report the error and stop |
| Stories use slug identifiers (not UUIDs) | Use the slug wherever an id is required; verify link creation explicitly (slugs may not resolve in all daemon versions) |
