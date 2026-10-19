---
name: future-man
description: This skill should be used when the user asks to "future-man this project", "envision the future of this project", "generate ideas for this project", "what could this project become", "vision this project", "seed ideas", "be the future man", or wants an AI agent to deeply analyze the project and generate forward-thinking ideas as Centy items.
allowed-tools: mcp__centy__centy_v1_CentyDaemon_IsInitialized mcp__centy__centy_v1_CentyDaemon_ListItemTypes mcp__centy__centy_v1_CentyDaemon_CreateItemType mcp__centy__centy_v1_CentyDaemon_ListItems mcp__centy__centy_v1_CentyDaemon_GetItem mcp__centy__centy_v1_CentyDaemon_CreateItem mcp__centy__centy_v1_CentyDaemon_CreateLink
---

# future-man

Become the visionary of any project. Future Man analyzes what exists — codebase structure, documentation, existing Centy items — and imagines what could exist next. It seeds a custom **`ideas`** item type and fills it with forward-thinking, context-grounded ideas that span quick wins to moonshots.

When the user gives a count, generate that many ideas. When the user gives no count, generate **7 ideas** by default.

---

## The `ideas` item type

`ideas` is a first-class Centy item type owned by Future Man. Each idea is a structured, trackable unit of project vision — not a vague wish, but a grounded proposal with full metadata.

### Custom fields

| Field | Type | Values |
|-------|------|--------|
| `category` | enum | `feature`, `improvement`, `architecture`, `integration`, `ux`, `performance`, `security`, `ecosystem` |
| `horizon` | enum | `quick-win`, `mid-term`, `long-term`, `moonshot` |
| `impact` | enum | `low`, `medium`, `high`, `transformative` |
| `inspiration` | string | What in the project prompted this idea |

### Statuses

`raw` → `promising` → `validated` → `shelved`

New ideas start as `raw`. The team promotes ideas as they gain confidence.

---

## Workflow

### 0 — Prerequisites

Ensure the daemon is running (start it if it is not) and the project is initialized. Resolve the project path from the current working directory.

---

### 1 — Ensure the `ideas` item type exists

List the item types for the project.

If no type with plural `"ideas"` exists, create it with these settings:

- Name `"Idea"`, plural `"ideas"`, identifier kind `"uuid"`.
- Features: display number on, status on, priority on, assets off, soft delete on.
- Statuses: `raw`, `promising`, `validated`, `shelved`; default status `raw`.
- Three priority levels.
- Four custom fields, all optional:
  - `category` (enum, values listed above)
  - `horizon` (enum, values listed above)
  - `impact` (enum, values listed above)
  - `inspiration` (string)

If the `ideas` type already exists, skip creation and continue with the existing type. If listing types fails, try creating the type anyway and continue if that succeeds.

---

### 2–3 — Research phase (run in a sub-agent)

Launch a **read-only sub-agent**. It must not create, update, or delete anything. Hand it the project path and the idea count (default 7, or the user's count).

The sub-agent researches in four steps and returns one research summary:

1. **Identify the project.** Read, in priority order, the README, the package manifest for the project's language (whichever exists), the changelog, and any top-level docs index. Extract the project name, a 2–4 sentence summary, the tech stack, and the likely user personas.
2. **Read the codebase structure.** List top-level files and directories, then one or two levels into the main source directories. Identify the architecture style, 3–6 key modules, and notable design patterns or absences.
3. **Read existing Centy items.** List the epics (up to 20), the issues with no status filter (up to 30), and all existing ideas. Extract the 3–5 most prominent open epics or issues as the current focus, the recurring themes, and the titles of all existing ideas so no duplicates are created.
4. **Synthesize opportunities.** Identify gaps no current epic or issue covers, leverage points, user pain points, emerging patterns in the tech landscape, and quick wins.

The research summary must contain every field above, as lists where the field is a list.

Wait for the research summary before continuing.

---

### 4 — Generate ideas

Using the research summary, generate exactly the requested number of ideas.

**Rules for idea generation:**

1. **Be specific and grounded** — every idea must reference something concrete from the research (a gap, a leverage point, a pain point, a module, a recurring theme). Generic ideas like "improve performance" are not acceptable — name *which* component, *which* bottleneck, and *why* it matters for this project specifically.
2. **Diversify** — the full set must span at least 3 different categories and at least 2 different horizons.
3. **No duplicates** — check the existing idea titles; do not generate any idea that substantially overlaps in title or concept with an existing one.
4. **Clear value proposition** — every idea must answer: *why would a user or developer care about this?*
5. **Moonshot balance** — include at least 1 moonshot idea that is ambitious but coherent with the project's trajectory.
6. **Quick-win balance** — include at least 1 quick-win idea that could realistically ship in a day or two.

**For each idea, define:** a concise action-oriented title (5–10 words), category, horizon, impact, a 1–2 sentence inspiration note naming what in the research prompted it, a priority (1 high, 2 medium, 3 low), 1–3 lowercase tags derived from the idea's themes, and a body that follows the template below.

**Idea body template:**

```markdown
## Vision

<2-3 sentences describing what this idea looks like when realized — concrete, not abstract>

## Why this project

<2-3 sentences grounded in the research — name the specific gap, leverage point, pain point, or pattern that makes this idea right for *this* project right now>

## Potential impact

- <Concrete outcome 1>
- <Concrete outcome 2>
- <Concrete outcome 3>
- <Add up to 2 more if meaningful>

## Possible approach

<2-3 sentences on how this could be implemented — not a full spec, but enough to show it is technically feasible within the existing architecture>

## Inspiration

<What in the codebase, existing items, or tech landscape directly inspired this idea>
```

---

### 5 — Create idea items

For each generated idea, in order, create an item of type `"ideas"` with:
- the idea title and body
- status `raw`
- the idea's priority and tags
- the four custom fields set to the idea's category, horizon, impact and inspiration text. Custom field values are JSON-encoded strings, so quote them and escape any inner quotes.

Record the created item's ID and display number. Continue to the next idea even if one creation fails; record failures as warnings.

---

### 6 — Report to the user

Print a visionary summary using this format:

```
Future Man — <project_name>

Seeded <idea_count> ideas into the ideas backlog.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  #<N>  <title>
        <horizon> · <category> · impact: <impact>
        <one-line teaser from the Vision section>
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  #<N>  <title>
        …
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Ideas start at status "raw". Promote any idea to "promising" when you're ready to
explore it, "validated" when you're committed to building it, or "shelved" if it
no longer fits the project's direction.
```

Surface any creation failures as warnings below the summary.

---

## Edge cases

| Situation | Handling |
|-----------|----------|
| `ideas` type already exists | Skip type creation; use the existing type as-is |
| Existing ideas are present | Read their titles from the item listing; avoid generating any duplicate |
| README or project files are missing | Rely on Centy items and codebase structure for research; note the absence |
| No open epics or issues in Centy | Base ideas entirely on codebase structure and files; note the lack of structured context |
| User specifies count < 3 | Generate at least 3 ideas; warn that fewer ideas limits diversity |
| User specifies count > 20 | Cap at 20; warn the user |
| Idea creation fails | Log as a warning; continue with remaining ideas |
| Listing item types fails | Attempt type creation anyway; continue if it succeeds |
| `ideas` type exists but is missing custom fields | Proceed without setting missing fields; note which fields were skipped |
| All requested ideas would duplicate existing ones | Generate the closest non-duplicate alternatives and note the substitution |
