---
name: future-man
description: This skill should be used when the user asks to "future-man this project", "envision the future of this project", "generate ideas for this project", "what could this project become", "vision this project", "seed ideas", "be the future man", or wants an AI agent to deeply analyze the project and generate forward-thinking ideas as Centy items.
allowed-tools: mcp__centy__IsRunning mcp__centy__StartDaemon mcp__centy__centy_v1_CentyDaemon_IsInitialized mcp__centy__centy_v1_CentyDaemon_ListItemTypes mcp__centy__centy_v1_CentyDaemon_CreateItemType mcp__centy__centy_v1_CentyDaemon_ListItems mcp__centy__centy_v1_CentyDaemon_GetItem mcp__centy__centy_v1_CentyDaemon_CreateItem mcp__centy__centy_v1_CentyDaemon_CreateLink
---

# future-man

Become the visionary of any project. Future Man analyzes what exists — codebase structure, documentation, existing Centy items — and imagines what could exist next. It seeds a custom **`ideas`** item type and fills it with forward-thinking, context-grounded ideas that span quick wins to moonshots.

When called with a count (e.g. `/future-man 5`), generate that many ideas. When called with no arguments, generate **7 ideas** by default.

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

New ideas are created with status `raw`. The team promotes ideas as they gain confidence.

---

## Workflow

### 0 — Prerequisites

Ensure the daemon is running and the project is initialized. Resolve `project_path` from the current working directory. Start the daemon with `StartDaemon` if `IsRunning` returns false.

---

### 1 — Ensure the `ideas` item type exists

Call `ListItemTypes` with `project_path`.

If a type with `plural: "ideas"` is **not** present, create it now:

```
CreateItemType:
  project_path: <project_path>
  name: "Idea"
  plural: "ideas"
  identifier: "uuid"
  features:
    display_number: true
    status: true
    priority: true
    assets: false
    soft_delete: true
  statuses: ["raw", "promising", "validated", "shelved"]
  default_status: "raw"
  priority_levels: 3
  custom_fields:
    - name: "category"
      field_type: "enum"
      required: false
      enum_values: ["feature", "improvement", "architecture", "integration", "ux", "performance", "security", "ecosystem"]
    - name: "horizon"
      field_type: "enum"
      required: false
      enum_values: ["quick-win", "mid-term", "long-term", "moonshot"]
    - name: "impact"
      field_type: "enum"
      required: false
      enum_values: ["low", "medium", "high", "transformative"]
    - name: "inspiration"
      field_type: "string"
      required: false
```

If `plural: "ideas"` already exists, skip creation and continue with the existing type.

---

### 2–3 — Research phase (run in a sub-agent)

Launch a **read-only sub-agent**. It must not create, update, or delete anything. Hand it:
- `project_path`
- `idea_count` (default 7, or the user-specified count)

**Sub-agent prompt template:**

```
You are performing the research phase of a "future-man" workflow.
Do NOT create, update, or delete anything — read only.

project_path: <project_path>
Ideas to generate: <idea_count>

Steps:

1. IDENTIFY PROJECT IDENTITY
   Search for and read (in order of priority):
   - README.md or README
   - package.json, Cargo.toml, pyproject.toml, go.mod, composer.json (whichever exists)
   - CHANGELOG.md or CHANGELOG
   - Any top-level docs/ or documentation/ index files

   Extract:
   - project_name: the name of the project
   - project_summary: 2-4 sentences on what it does and who it is for
   - tech_stack: languages, frameworks, key dependencies
   - user_personas: inferred types of users (e.g., "developer building CLI tools", "team lead managing epics")

2. READ CODEBASE STRUCTURE
   Use Glob to list top-level files and directories, then 1-2 levels deep into
   the main source directories (e.g., src/, lib/, app/, daemon/, cli/, etc.).

   Identify:
   - architecture_style: e.g., "gRPC daemon + MCP plugin", "REST API + React SPA", "CLI tool"
   - key_modules: notable subsystems or areas of the codebase (list 3-6)
   - technical_observations: interesting design patterns, notable absences, or architectural choices

3. READ EXISTING CENTY ITEMS
   a. Call ListItems (item_type: "epics") — up to 20 items
   b. Call ListItems (item_type: "issues") with no status filter — up to 30 items
   c. Call ListItems (item_type: "ideas") — read ALL existing ideas to avoid duplicates

   Extract:
   - current_focus: summarize the 3-5 most prominent open epics or issues in plain language
   - recurring_themes: patterns that appear across multiple items (e.g., "performance", "auth", "CLI UX")
   - existing_idea_titles: list titles of ALL existing ideas — you must not create duplicates

4. SYNTHESIZE OPPORTUNITIES
   Based on everything above, identify the richest opportunities for the project:
   - gaps: areas the project could logically expand into that no current epic or issue covers
   - leverage_points: existing strengths that could be amplified further
   - user_pain_points: problems the user personas likely face that the project does not yet solve
   - emerging_patterns: trends in the tech landscape relevant to this project's domain
   - quick_wins: small, high-leverage improvements that could have outsized impact

Return a single RESEARCH block:

RESEARCH:
  project_name: <string>
  project_summary: <string>
  tech_stack: [<string>, …]
  user_personas: [<string>, …]
  architecture_style: <string>
  key_modules: [<string>, …]
  current_focus: [<string>, …]
  recurring_themes: [<string>, …]
  existing_idea_titles: [<string>, …]
  opportunities:
    gaps: [<string>, …]
    leverage_points: [<string>, …]
    user_pain_points: [<string>, …]
    emerging_patterns: [<string>, …]
    quick_wins: [<string>, …]
```

Wait for the RESEARCH block before continuing.

---

### 4 — Generate ideas

Using the RESEARCH block, generate exactly `<idea_count>` ideas.

**Rules for idea generation:**

1. **Be specific and grounded** — every idea must reference something concrete from the RESEARCH (a gap, a leverage point, a pain point, a module, a recurring theme). Generic ideas like "improve performance" are not acceptable — name *which* component, *which* bottleneck, and *why* it matters for this project specifically.
2. **Diversify** — the full set must span at least 3 different `category` values and at least 2 different `horizon` values.
3. **No duplicates** — check `existing_idea_titles`; do not generate any idea that substantially overlaps in title or concept with an existing one.
4. **Clear value proposition** — every idea must answer: *why would a user or developer care about this?* Either "what problem does this solve?" or "what new capability does this unlock?"
5. **Moonshot balance** — include at least 1 `moonshot` idea that is ambitious but coherent with the project's trajectory.
6. **Quick-win balance** — include at least 1 `quick-win` idea that could realistically be shipped in a day or two.

**For each idea, define:**

```
Idea <N>:
  title: <concise, action-oriented, 5–10 words>
  category: <feature | improvement | architecture | integration | ux | performance | security | ecosystem>
  horizon: <quick-win | mid-term | long-term | moonshot>
  impact: <low | medium | high | transformative>
  inspiration: <1-2 sentences: what in the RESEARCH prompted this idea>
  priority: <1 = high, 2 = medium, 3 = low>
  tags: [<1-3 lowercase tags derived from the idea's themes>]
  body: <see template below>
```

**Idea body template:**

```markdown
## Vision

<2-3 sentences describing what this idea looks like when realized — concrete, not abstract>

## Why this project

<2-3 sentences grounded in the RESEARCH — name the specific gap, leverage point, pain point, or pattern that makes this idea right for *this* project right now>

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

For each generated idea, in order:

1. Call `CreateItem`:
   - `project_path`: `<project_path>`
   - `item_type`: `"ideas"`
   - `title`: the idea title
   - `body`: the idea body (filled from template)
   - `status`: `"raw"`
   - `priority`: the idea priority
   - `custom_fields`:
     - `"category"`: `"\"<category>\""` (JSON-encoded string)
     - `"horizon"`: `"\"<horizon>\""` (JSON-encoded string)
     - `"impact"`: `"\"<impact>\""` (JSON-encoded string)
     - `"inspiration"`: `"\"<inspiration text>\""` (JSON-encoded string — escape inner quotes)
   - `tags`: the idea's tags array

2. Record the created item's `id` and `display_number`.

Continue to the next idea even if one creation fails — record failures as warnings.

---

### 6 — Report to the user

Print a visionary summary using this format:

```
Future Man — <project_name>

Seeded <idea_count> ideas into the ideas backlog.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  #<N>  <title>
        <horizon> · <category> · impact: <impact>
        <one-line teaser from the Vision section>
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  #<N>  <title>
        …
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Ideas start at status "raw". Promote any idea to "promising" when you're ready to
explore it, "validated" when you're committed to building it, or "shelved" if it
no longer fits the project's direction.
```

Surface any creation failures as warnings below the summary.

---

## Edge cases

| Situation | Handling |
|-----------|----------|
| `ideas` type already exists | Skip `CreateItemType`; use the existing type as-is |
| Existing ideas are present | Read their titles from `ListItems`; avoid generating any duplicate |
| README or project files are missing | Rely on Centy items and codebase structure for research; note the absence |
| No open epics or issues in Centy | Base ideas entirely on codebase structure and files; note the lack of structured context |
| User specifies count < 3 | Generate at least 3 ideas; warn that fewer ideas limits diversity |
| User specifies count > 20 | Cap at 20; warn the user |
| Idea creation fails | Log as a warning; continue with remaining ideas |
| `ListItemTypes` fails | Attempt `CreateItemType` anyway; continue if it succeeds |
| `ideas` type exists but is missing custom fields | Proceed without setting missing fields; note which fields were skipped |
| All requested ideas would duplicate existing ones | Generate the closest non-duplicate alternatives and note the substitution |
