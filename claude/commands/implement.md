You are implementing a design. Your goal is to turn an approved design document into working code using parallel task execution.

## Input

The user has an approved design document (from /design). They may provide:
- A path to the design document
- A reference to the most recent design
- Just "/implement" if the design is in conversation context

If no design is available, stop and tell the user to run /design first.

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Create Beads task | No — main agent | Track work before starting |
| 2. Detect mode | No — main agent | Determines Mode A or Mode B |
| 3. Read/create task graph | No — main agent | Mode A reads graph; Mode B creates it |
| 4. Create shared interfaces | No — main agent | Must exist before workers start |
| 5. Spawn workers | Yes — all workers | Independent implementation tasks |
| 6. Integrate | No — main agent | Verify changes, run tests |
| 7. Archive consumed design | No — main agent | Move design to finished/ in cockpit |
| 8. Close tasks and report | No — main agent | Summarizes results with ## Tracking |

## Process

### 1. Create Beads task

```bash
bd_id=$(krust bd-start task "Implement: [design name]")
```

Under krust, `krust bd-start` prints `$KRUST_BEADS_ID` (no-op). Standalone, it creates + claims a new task.

**CRITICAL**: `$bd_id` is your orchestration task for this entire run. Do NOT create a separate orchestration task via `bd create`. Use `$bd_id` in the implementation report frontmatter and in `krust bd-finish`. The only tasks you create via `bd create` are child implementation tasks under the epic.

### 2. Detect mode

Check if a task graph already exists (from `/plan`):

```
bd list --parent=<epic-matching-design> --status=open
```

- **If results exist** → Mode A (task graph exists, proceed to Step 3A)
- **If empty** → Mode B (no task graph, proceed to Step 3B)

If `$KRUST_BEADS_ID` is set, `$bd_id` is the epic id:
```bash
epic_id="$bd_id"
```

Standalone: discover the epic from the design reference or `$ARGUMENTS` (existing behavior).

### 3A. Mode A — Read existing task graph

The task graph was created by `/plan`. Read it:

```
bd list --parent=<epic-id> --status=open --pretty
```

Identify which tasks are ready (no unresolved deps) and which are blocked. Skip to Step 5.

### 3B. Mode B — Create task graph inline

Parse the design document and extract:
- **Components** to build (from Architecture section)
- **Interfaces** to implement (from Interfaces section)
- **Data schemas** to create/modify (from Data Schemas section)
- **Dependencies** between tasks (which tasks block others)

If `$KRUST_BEADS_ID` is set and `bd list --parent=<epic_id> --status=open` returns no results, report an error:
> "Epic <id> has no open tasks. Run /plan first, or check if tasks are already closed."

Create Beads tasks for each component: `bd create --title="..." --description="..." --type=task --labels=<resolved-labels>` with interface spec, file scope, and acceptance criteria in the description. Wire deps with `bd dep add`.

### 4. Create shared interfaces

Before spawning workers, create any shared types, interfaces, or proto definitions that multiple workers need. This prevents workers from inventing incompatible interfaces.

Write these shared files before spawning workers.

### 5. Spawn workers (parallel)

For each ready task (no unresolved deps), spawn a worker:

**Worker** (Agent, model=opus):
> You are implementing one task from a design.
>
> ## Your Task: [task-id]
> [paste task description from Beads, including interface spec and file scope]
>
> ## Interface Spec
> [paste the relevant interfaces from the design]
>
> ## Files
> - Modify: [file paths]
> - Create: [file paths]
>
> ## Rules
> - Claim your Beads task before starting: `bd update [task-id] --claim`
> - Follow the interface spec exactly — do not add methods, fields, or parameters not in the spec
> - Match existing code patterns in the repo (error handling, naming, structure)
> - Write tests for the code you write
> - Do not modify files outside your task scope
> - Do not run any git commands (no commits, no branching, no stashing)
> - If you're blocked or find the design is ambiguous, report the issue — do not guess
> - If you discover work outside your task scope (missing APIs, tech debt, schema gaps), report it at the end of your output:
>   ```
>   ## Discovered Work
>   - [title]: [one-line description]
>   ```
>   Do not create beads tasks yourself — the orchestrator will file them

Spawn ALL ready workers in ONE message.

For tasks with unresolved deps: wait for blocking workers to complete, then spawn the next wave.

### 6. Integrate

After all workers complete:

- Review the changes made by all workers for consistency
- Run any existing tests (`go test ./...`, `npm test`, `pytest`, etc.)
- Fix integration issues — these are usually import paths, type mismatches, or missing glue code
- Collect `## Discovered Work` sections from all worker outputs
- Create beads tasks for discovered items as **top-level tasks** (NOT children of the epic) with a dependency on the epic. This keeps the epic eligible for auto-close while still surfacing the work in `bd ready` once the epic closes:
  ```bash
  new_id=$(bd create --title="[title]" --description="[description]" --type=task --labels=<resolved-labels>)
  bd dep add "$new_id" <epic-id>
  ```
- Do not run any git commands — krust owns git operations

If any check fails, fix it before reporting.

### 7. Archive consumed design

If the design document lives in `$COCKPIT_DIR/state/designs/`, you'll archive it in Step 8 — after `krust artifact implementations` runs, so the archive reason can use the resolved implementation slug (which may differ from the input slug if a collision was auto-bumped to `-v2`..`-v5`). If the design wasn't from the cockpit, no archive is needed.

### 8. Close tasks and report

For each completed task: `bd close <task-id>`

Write the Implementation Report (use `$bd_id` from Step 1 — do NOT use any other task ID here):

```markdown
---
beads_id: <$bd_id from Step 1>
---

# Implementation Report: [Design Name]

## Tasks Completed
- <task-id>: <title> — closed

## Files Created/Modified
[list files with brief description]

## Test Results
[pass/fail summary]

## Deviations & Discovered Work
[differences from design, with rationale]
[newly created task IDs for discovered work, or "None"]

## Tracking
- Orchestration: <$bd_id from Step 1> — closed
- Epic: <epic-id>
```

Use the Write tool to save the report as a file at `$KRUST_OUT` (if set) or `/tmp/implement-report-<bd_id>.md`. You MUST write the file to disk — do not just output the report as conversation text.

Then run these commands via Bash:

```bash
# Emit the implementation artifact and capture the resolved slug. The resolved
# slug may differ from the input if there was a collision (auto-bumped to -v2..-v5).
result=$(krust artifact implementations "$slug" "$report_path")
final_slug=$(echo "$result" | jq -r .slug)

if [ "$final_slug" != "$slug" ]; then
  echo "Slug collided; resolved to $final_slug"
fi

# Archive the consumed design (Step 7) using the resolved implementation slug, so
# the archive reason matches the actual implementation-report filename. Skip if
# the design wasn't in $COCKPIT_DIR/state/designs/.
krust archive designs "$design_file" "implemented: $final_slug"

krust bd-finish "$bd_id"
```

`krust artifact` declares the report as an output artifact and prints a JSON line on stdout with the resolved `slug` and `path`. `krust bd-finish` closes the orchestration task (no-op under krust — the wrapper closes it on approval).

## Rules

- **Do not start without a design** — if there's no design document, stop
- **Skills do not run git directly** — krust owns all git operations. Do not commit, branch, stash, merge, or run any git commands.
- **No worktrees** — do not use `isolation: "worktree"` on worker agents
- **Shared interfaces first** — create them before spawning workers to prevent drift
- **Workers follow the spec exactly** — no freelancing, no extra methods, no bonus abstractions
- **Match existing patterns** — look at neighboring code for conventions before writing new code
- **Tests are required** — every worker writes tests for its code
- **Report deviations** — if implementation must differ from design, explain why
- **## Tracking is mandatory** — output must include Beads task IDs
