---
name: gap-analyze
description: This skill should be used when the user asks to "analyze gaps in an epic", "check user story coverage", "gap analyze epic", "make sure an epic has user stories", "validate stories for an epic", or wants to ensure an epic's goals are fully covered by well-formed user stories.
allowed-tools: mcp__centy__centy_v1_CentyDaemon_GetDaemonInfo mcp__centy__centy_v1_CentyDaemon_IsInitialized mcp__centy__centy_v1_CentyDaemon_ListItems mcp__centy__centy_v1_CentyDaemon_GetItem mcp__centy__centy_v1_CentyDaemon_ListLinks mcp__centy__centy_v1_CentyDaemon_ListItemTypes mcp__centy__centy_v1_CentyDaemon_CreateItem mcp__centy__centy_v1_CentyDaemon_UpdateItem mcp__centy__centy_v1_CentyDaemon_CreateLink
---

# gap-analyze

Inspect an epic's stated goals, find any areas not covered by a well-formed user story, create or update stories to fill those gaps, then flag any story that still fails the quality bar by raising an issue.

When the user gives an epic ID or display number, operate on that epic. When the user gives no epic, auto-discover all open or in-progress epics in the project and process each one in turn.

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
Try to call the `GetDaemonInfo` tool. If the `mcp__centy__*` tools are not available (not in the tool list, or a tool-not-found error), fall back to the `centy` command-line tool for every read and write action in this workflow. Note the degraded mode in the final report. Do not abort — continue with the CLI fallback.

**b. Check version compatibility.**
After confirming the daemon is reachable, compare the project version against the daemon version (the `GetDaemonInfo` tool or the CLI config command reports both). If they differ, stop and tell the user:

> Version mismatch detected: project is at X, daemon is at Y.
> Run the centy init command to migrate, then re-run gap-analyze.

Do not proceed past step 0 until versions match.

**c. Start the daemon if needed.**
If the daemon is not running, start it. Resolve the project path from the current working directory. Confirm the project is initialized (`IsInitialized`) before continuing.

---

### 1–3 — Research phase (run in a sub-agent)

Launch a **read-only sub-agent**. It must not create, update, or delete anything. Hand it:
- the epic ID or display number if one was provided; otherwise a "discover" marker
- the project path
- whether MCP tools are available (from step 0a)

If MCP is not available, the sub-agent uses the CLI for all reads.

The sub-agent works in four steps and returns one report per epic:

0. **Resolve the epic(s).** If an ID or display number was given, fetch that epic. If discovering, list the epics and select those whose status is open or in-progress. If none exist, select all epics regardless of status. Process each epic independently through steps 1–4.
1. **Read the epic fully.** Capture its ID, display number, title, body, status, tags and custom fields.
2. **Extract intended scope from the epic body.** If the body has explicit Done and Pending sections (or Complete and Remaining, or the equivalent headings), extract scope items ONLY from the Pending section; Done items are already shipped and need no story coverage. Otherwise, parse the full body for lists of features, headings naming areas of functionality, sentences such as "the system should", "users should be able to", "support", "enable", "allow" or "as a", and acceptance criteria blocks. Each becomes a scope item. If there are more than 5 pending scope items, group related items by theme into at most 3 thematic groups, each treated as one scope item with its constituent items listed under it. If the body is empty or too vague, record a single scope item saying no structured scope was found and a generic story is needed.
3. **Discover linked user stories.** List the links for the epic. For each link, read its link type and the item type of the other end, verbatim from the link record. Only treat an item as a story when the link is a parent-of link to a story (epic to story), or a child-of link from a story (story to epic). Do not classify items of other types, such as issues or docs, as stories, even if they appear in a parent-of link. For each qualifying story, fetch it as a story item using its identifier, and record its ID, title, body and status (an empty status counts as `draft`). Classify each story as PASS (meets all four passing criteria) or FAIL (violates one or more, listing which).
4. **Map coverage.** For each scope item or thematic group, mark it COVERED if at least one linked story clearly addresses it, or UNCOVERED if none does.

Each report contains: the epic's ID, display number and title; the scope items, each with its text, any grouped constituent items, whether it is covered, and the IDs of covering stories; the existing stories, each with ID, title, status, result (pass or fail) and fail reasons; and a warnings list.

Wait for all reports before continuing. If the sub-agent returned multiple reports (discover mode), process them one at a time: complete steps 4–8 for the first epic, then move on to the next — no user input between epics.

---

### 4 — Plan story changes

Using the report, build an action list:

**For each UNCOVERED scope item (or thematic group):**
- Check if any existing story (even one that failed) partially covers it.
  - If yes → mark that story for **UPDATE** with the uncovered scope merged in.
  - If no → plan to **CREATE** one new story covering the entire scope item or group.

**For each existing story with result `fail`:**
- If it is already marked for UPDATE → the update will also fix the failing fields.
- If it is not yet in the update list → plan to **UPDATE** it to meet the passing criteria.

---

### 5 — Create new stories

For each planned CREATE, write the story body using the template below.

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
1. Create an item of type `"stories"` with the title, the filled body, and status `ready`. With the CLI fallback, use the CLI's create-story command with the same values.
2. Create a link from the epic to the new story:
   - source: the epic's ID, source item type `"epic"`
   - target: the new story's ID (UUID or slug), target item type `"story"`
   - link type: `"parent-of"`
   - With the CLI fallback, use the CLI's link option on the epic's update command.
3. **Verify the link was actually created.** List the story's links and confirm the epic appears. If the link is absent, log a warning: "Link creation reported success but link was not found — story `<id>` is unlinked." Do not raise an issue for this; it is an infrastructure bug, not a story quality failure.
4. Verify the story passes all four criteria. If it does, mark it `result: "pass"`.

---

### 6 — Update existing stories

For each planned UPDATE, merge in any missing content:
- If missing the user-story statement → prepend it above the existing body.
- If missing acceptance criteria → append an `## Acceptance criteria` section with generated criteria based on the scope item and epic context.
- If status is `"draft"` or empty → set status to `"ready"`.

Steps:
1. Update the story item (type `"stories"`) with the revised body and/or status. With the CLI fallback, use the CLI's update command for stories.
2. Re-evaluate the story against the four passing criteria.
3. If the story now passes, mark `result: "pass"`. If it still fails, mark `result: "fail"` and record the remaining fail reasons.

---

### 7 — Create issues for stories that still fail

For each story that still has `result: "fail"` after steps 5–6, create one issue of type `"issues"` with status `open` and priority 2.

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

Then link the new issue to the story: source is the issue's ID with item type `"issue"`, target is the story's ID with item type `"story"`, link type `"blocks"`.

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
| MCP tools unavailable | Fall back to the `centy` CLI; note degraded mode in report |
| Version mismatch (project vs daemon) | Stop at step 0; tell user to run the centy init command |
| No epic given | Auto-discover all open/in-progress epics; process each one |
| Epic body is empty | Generate one generic story covering the epic title; note the lack of scope detail |
| Epic has Done/Pending sections | Extract scope only from Pending; ignore Done items |
| More than 5 pending scope items | Group by theme into ≤ 3 thematic groups; create one story per group |
| Epic has no linked stories at all | Create one story per scope item/group (or one generic story if scope is empty) |
| All stories already pass | Report "fully covered — no changes needed"; skip steps 5–7 |
| A story passes after update | Do not raise an issue for it |
| A scope item maps to multiple stories | Mark as COVERED; no new story needed |
| Link verification fails after create/update | Log as warning; do not raise a quality issue |
| Story type not available in project | Fall back to the issues type and note the fallback in the report |
| Epic not found | Report the error and stop |
| Stories use slug identifiers (not UUIDs) | Use the slug wherever an id is required; verify link creation explicitly (slugs may not resolve in all daemon versions) |
