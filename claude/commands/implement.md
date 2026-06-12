You are implementing a design. Your goal is to turn an approved design document into working code using parallel task execution.

## Input

The brief comes from `$ARGUMENTS`. The user has an approved design document (from /design). They may provide:
- A path to the design document
- A reference to the most recent design
- Just "/implement" if the design is in conversation context
- A free-form brief describing the work — but only when `$ARGUMENTS` **explicitly states the brief is the spec**. The trigger is the explicit statement, NOT the mere presence of free-form text in `$ARGUMENTS`. A user typing a one-liner (e.g. `/implement add retry logic to the fetcher`) is NOT supplying a brief-as-spec — that is the no-design case below, which still stops. When the explicit statement is present, treat the brief as the design source and create the task graph inline (Mode B).

If no design is available, stop and tell the user to run /design first — unless `$ARGUMENTS` **explicitly stated the brief is the spec** (the bullet above), in which case use that brief as the design source and do not stop for a missing design doc. Even with the explicit statement, first apply a **minimum-sufficiency check**: the brief must name at least one concrete component or behavior to build AND enough detail to derive the interfaces/work for Step 3B. If it is too vague to build a task graph from (e.g. a bare one-line directive with no specifics), stop and tell the user to run /design first rather than fabricating a graph from an underspecified input.

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Create Beads task | No — main agent | Track work before starting |
| 2. Detect mode | No — main agent | Determines Mode A or Mode B |
| 3. Read/create task graph | No — main agent | Mode A reads graph; Mode B creates it |
| 4. Create shared interfaces | No — main agent | Must exist before workers start |
| 5. Spawn workers | Yes — all workers | Independent implementation tasks |
| 6. Integrate | No — main agent | Verify changes, run tests |
| 7. Tighten changes | No — main agent | /simplify + comment audit, re-run tests |
| 8. Archive consumed design | No — main agent | Move design to finished/ in cockpit |
| 9. Close tasks and report | No — main agent | Summarizes results with ## Tracking |

## Process

### 1. Create Beads task

If `$ARGUMENTS` supplies an epic/task id, use it as the orchestration task and claim it:

```bash
bd update <id> --claim
bd_id=<id>
```

Otherwise create and claim a new orchestration task:

```bash
bd_id=$(bd create --title="Implement: [design name]" --type=task)
bd update "$bd_id" --claim
```

**CRITICAL**: `$bd_id` is your orchestration task for this entire run. Do NOT create a separate orchestration task via `bd create` once you already have one. Use `$bd_id` in the implementation report frontmatter and when finishing the run (Step 9). The only tasks you create via `bd create` are child implementation tasks under the epic.

### 2. Detect mode

**First, check the brief-as-spec signal.** If `$ARGUMENTS` explicitly stated the brief is the spec (see Input) and the brief passes the minimum-sufficiency check there, this is **Mode B unconditionally**: there is no design doc, so the `bd list --parent=<epic>` query below is undefined and MUST be skipped. Proceed straight to Step 3B and build the task graph inline from the brief. This precedence rule resolves the conflict where a brief is supplied but a planned task graph also happens to exist — the brief wins, and you do NOT build a duplicate inline graph atop the planned one.

Otherwise (a design doc exists), check whether a task graph already exists (from `/plan`):

```
bd list --parent=<epic-matching-design> --status=open
```

- **If results exist** → Mode A (task graph exists, proceed to Step 3A)
- **If empty** → Mode B (no task graph, proceed to Step 3B)

Bind the epic id for the rest of the run:

```bash
epic_id="$bd_id"
```

If `$ARGUMENTS` did not supply an epic id, discover the epic from the design reference instead.

### 3A. Mode A — Read existing task graph

The task graph was created by `/plan`. Read it:

```
bd list --parent=<epic-id> --status=open --pretty
```

Identify which tasks are ready (no unresolved deps) and which are blocked.

**Load the plan artifact.** The plan file path is recorded in beads metadata — read `artifact_path` from `bd show <epic-id> --json` and read that plan.

### 3B. Mode B — Create task graph inline

Parse the spec and extract the same four things, adapting to its shape:
- **Components** to build — from the Architecture section of a structured design doc; from a free-form brief, infer them from the work it describes
- **Interfaces** to implement — from the Interfaces section; for a free-form brief, derive the signatures/types the work implies (spec them explicitly even when the brief is prose)
- **Data schemas** to create/modify — from the Data Schemas section; absent in a free-form brief, so include only schemas the brief actually requires
- **Dependencies** between tasks (which tasks block others)

If an epic id was supplied and `bd list --parent=<epic_id> --status=open` returns no results, report an error — **unless `$ARGUMENTS` supplied a brief as the spec**, in which case no pre-planned tasks is the expected state (which is precisely why it's Mode B), so skip the error and create the task graph inline below:
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

Spawn ALL ready workers in ONE assistant message using the `Agent` tool. Each is a synchronous, blocking call — multiple `Agent` tool uses in a single message run concurrently and the harness blocks the turn until every `tool_result` returns. **Do not set `run_in_background: true`. Do not use `TeamCreate` or any team-lifecycle tools** — async/teams semantics cause sub-agent completions to arrive as `task_notification` events that the lead can narrate and end its turn on without writing the artifact.

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

If any check fails, fix it before reporting.

### 7. Tighten changes

After integration is green, tighten the code the workers produced before it leaves the skill. There is no PR yet, so everything here is scoped to the implementation's **working-tree changes** (the uncommitted diff).

1. **Simplify the code.** Invoke the `/simplify` skill over the changes. It applies reuse, simplification, efficiency, and altitude cleanups in place — collapsing duplication, swapping reinvented helpers for existing ones, and removing complexity the parallel workers couldn't see across task boundaries.

2. **Audit comments for load-bearing value.** Review every comment the implementation added or changed and cut the ones that don't earn their place:
   - **Always strip personal workflow artifacts** (no judgment call — these must never ship in code): beads task IDs (e.g. `jmcfarland-gzr4`), links or paths to design docs (`$COCKPIT_DIR/state/designs/…` or any cockpit reference), "see the plan / per the design" pointers, and any other internal tracking reference. They are meaningless to a repo reader and leak private context.
   - **Remove** comments that explain *what* well-named code already says, that reference the task/PR ("added for X"), or that are tombstones (`# renamed from…`, stale TODOs).
   - **Collapse** multi-paragraph docstrings or blocks to the one line that carries the load.
   - **Keep** only the non-obvious *why*: hidden constraints, invariants, ordering requirements, workaround-for-bug notes. When in doubt, keep — deleting load-bearing context is worse than leaving a marginal comment.

3. **Re-run the Step 6 tests.** Both passes edit real code, so re-run the same tests from Step 6. If a cleanup broke something, fix it or revert that specific change — never report with red tests.

### 8. Archive consumed design

If the design document lives in `$COCKPIT_DIR/state/designs/`, move it to `$COCKPIT_DIR/state/designs/finished/` (run `mkdir -p "$COCKPIT_DIR/state/designs/finished"` first if needed). Use the implementation `$slug` in the commit message. If the design wasn't from the cockpit, no archive is needed.

### 9. Close tasks and report

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

Pick a `<slug>` — a kebab-case version of the design/feature name (max 50 chars). Use the Write tool to save the report to `$COCKPIT_DIR/state/implementations/implementations-<slug>.md` (run `mkdir -p "$COCKPIT_DIR/state/implementations"` first if needed). You MUST write the file to disk — do not just output the report as conversation text.

Then publish and finish via Bash:

```bash
report_path="$COCKPIT_DIR/state/implementations/implementations-$slug.md"

# Commit and push the implementation report.
git -C "$COCKPIT_DIR" add "$report_path"
git -C "$COCKPIT_DIR" commit -m "implement: $slug"

# Archive the consumed design (Step 8) — skip if the design wasn't in
# $COCKPIT_DIR/state/designs/.
git -C "$COCKPIT_DIR" add "$design_file"  # the moved design (now under finished/)
git -C "$COCKPIT_DIR" commit -m "archive design: implemented $slug"

git -C "$COCKPIT_DIR" push

# Close the orchestration epic.
bd close "$bd_id"
```

## Rules

- **Do not start without a design** — if there's no design document, stop — unless `$ARGUMENTS` supplied a brief as the spec (see Input), in which case use that brief as the design source and proceed
- **Workers do not run git** — only the main agent commits/pushes the report and archived design (Step 9). Workers must not commit, branch, stash, or merge.
- **No worktrees** — do not use `isolation: "worktree"` on worker agents
- **Shared interfaces first** — create them before spawning workers to prevent drift
- **Workers follow the spec exactly** — no freelancing, no extra methods, no bonus abstractions
- **Match existing patterns** — look at neighboring code for conventions before writing new code
- **Tests are required** — every worker writes tests for its code
- **Report deviations** — if implementation must differ from design, explain why
- **## Tracking is mandatory** — output must include Beads task IDs
