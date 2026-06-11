You are implementing a design. Your goal is to turn an approved design document into working code using parallel task execution.

## Input

The user has an approved design document (from /design). They may provide:
- A path to the design document
- A reference to the most recent design
- Just "/implement" if the design is in conversation context
- A free-form brief describing the work — but only when the invoking prompt **explicitly states the brief is the spec** (graph mode), parallel to how Steps 7 and 9 require the prompt to explicitly say simplification or publication runs as a graph node. The trigger is the explicit statement, NOT the mere presence of free-form text in `$ARGUMENTS`. A standalone user typing a one-liner (e.g. `/implement add retry logic to the fetcher`) is NOT supplying a brief-as-spec — that is the no-design oneshot case below, which still stops. When the explicit statement is present, treat the brief as the design source and create the task graph inline (Mode B).

If no design is available, stop and tell the user to run /design first — unless the invoking prompt **explicitly stated the brief is the spec** (the bullet above), in which case use that brief as the design source and do not stop for a missing design doc. Even with the explicit statement, first apply a **minimum-sufficiency check**: the brief must name at least one concrete component or behavior to build AND enough detail to derive the interfaces/work for Step 3B. If it is too vague to build a task graph from (e.g. a bare one-line directive with no specifics), stop and tell the user to run /design first rather than fabricating a graph from an underspecified input.

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
| 8. Archive consumed design | No — main agent | Move design to finished/ in cockpit (always /implement, even in graph mode) |
| 9. Close tasks and report | No — main agent | Summarizes results with ## Tracking |

## Process

### 1. Create Beads task

```bash
bd_id=$(krust bd-start task "Implement: [design name]")
```

Under krust, `krust bd-start` prints `$KRUST_BEADS_ID` (no-op). Standalone, it creates + claims a new task.

**CRITICAL**: `$bd_id` is your orchestration task for this entire run. Do NOT create a separate orchestration task via `bd create`. Use `$bd_id` in the implementation report frontmatter and in `krust bd-finish`. The only tasks you create via `bd create` are child implementation tasks under the epic.

### 2. Detect mode

**First, check the brief-as-spec signal.** If the invoking prompt explicitly stated the brief is the spec (see Input — the graph-mode "brief-as-spec" variant) and the brief passes the minimum-sufficiency check there, this is **Mode B unconditionally**: there is no design doc, so the `bd list --parent=<epic-matching-design>` query below is undefined and MUST be skipped. Proceed straight to Step 3B and build the task graph inline from the brief. This precedence rule resolves the conflict where a brief is supplied but a planned task graph also happens to exist — the brief wins, and you do NOT build a duplicate inline graph atop the planned one.

Otherwise (a design doc exists), check whether a task graph already exists (from `/plan`):

```
bd list --parent=<epic-matching-design> --status=open
```

- **If results exist** → Mode A (task graph exists, proceed to Step 3A)
- **If empty** → Mode B (no task graph, proceed to Step 3B)

> **Note on "graph mode":** the label names two distinct krust-graph signals that select *different* modes. The **brief-as-spec** signal (a free-form brief supplied as the spec) selects Mode B, per the rule above. The **plan-path** signal in Step 3A (`stack_id` / `stack_layer` + explicit plan path) is a Mode A variant — it presupposes a plan already exists. A generic, unqualified "graph mode" signal therefore selects the brief-as-spec → Mode B reading here, and the plan-path → Mode A reading only when those three plan values are supplied.

If `$KRUST_BEADS_ID` is set, `$bd_id` is the epic id:
```bash
epic_id="$bd_id"
```

Standalone: discover the epic from the design reference or `$ARGUMENTS` (existing behavior).

**Stack-mode env vars** (set by krust when implementing a layer of a stacked plan):

- `KRUST_EPIC_ID` — the layer's epic ID (existing semantics; same as `$bd_id` under krust)
- `KRUST_STACK_ID` — the parent stack slug. Absent for single-layer plans.
- `KRUST_STACK_LAYER` — the layer position as integer (1-indexed). Absent for single-layer plans.

When `KRUST_STACK_ID` is unset, behavior is identical to single-mode (no plan-loading changes, no action-shape changes). When set, Step 3A loads the full plan file and anchors on the layer subsection — see below.

### 3A. Mode A — Read existing task graph

The task graph was created by `/plan`. Read it:

```
bd list --parent=<epic-id> --status=open --pretty
```

Identify which tasks are ready (no unresolved deps) and which are blocked.

**Load the plan artifact.** The plan file lives at the path recorded in beads metadata. Lookup precedence:

1. **Graph mode** (invoking prompt supplies `stack_id` / `stack_layer` and the full plan path explicitly): use those three values directly — they replace both the `KRUST_STACK_ID` / `KRUST_STACK_LAYER` env vars and the parent-task `artifact_path` lookup below. The stack-mode full-plan-load + layer-anchor invariant is otherwise unchanged: read the FULL plan at the supplied path and anchor on `### Layer <stack_layer>`.
2. **Stack mode** (`KRUST_STACK_ID` set): find the PARENT task that owns the stack slug — the task whose `KrustMeta.stack_slug == $KRUST_STACK_ID` — and read its `artifact_path`. Do NOT use the layer epic's `artifact_path`; layer epics may not carry the plan path themselves.
3. **Single mode** (`KRUST_STACK_ID` unset): read `artifact_path` from `bd show <epic-id> --json` for the current epic.

In **stack mode and graph mode** (precedence entries 1–2), /implement MUST read the FULL `plans-<slug>.md` (terminal state + every `### Layer N: ...` section), not just the section for the current layer. This is an invariant: terminal-state interfaces and sibling-layer contracts must stay in context so workers make correct cross-layer decisions. After loading the full file, anchor on `### Layer <layer>: <layer-slug>` — where `<layer>` is `KRUST_STACK_LAYER` in stack mode or the supplied `stack_layer` in graph mode — to extract THIS layer's task graph and merge-alone invariant. Also read the `## Stack Status` table — base branch references and prior-layer status are informational (krust handles base branch derivation at ship time).

> /implement always reads the full plan in stack mode and graph mode — never just its own slice.

In single mode, read the plan as today (no full-file invariant).

Skip to Step 5.

### 3B. Mode B — Create task graph inline

Parse the spec and extract the same four things, adapting to its shape:
- **Components** to build — from the Architecture section of a structured design doc; from a free-form brief, infer them from the work it describes
- **Interfaces** to implement — from the Interfaces section; for a free-form brief, derive the signatures/types the work implies (spec them explicitly even when the brief is prose)
- **Data schemas** to create/modify — from the Data Schemas section; absent in a free-form brief, so include only schemas the brief actually requires
- **Dependencies** between tasks (which tasks block others)

If `$KRUST_BEADS_ID` is set and `bd list --parent=<epic_id> --status=open` returns no results, report an error — **unless the invoking prompt supplied a brief as the spec (graph mode)**, in which case this is the expected state (a graph-mode brief run is krust-invoked with no pre-planned tasks, which is precisely why it's Mode B), so skip the error and create the task graph inline below:
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
- Do not run any git commands — krust owns git operations

If any check fails, fix it before reporting.

### 7. Tighten changes

**In graph mode** (the invoking prompt says simplification runs as a separate graph node), **skip only sub-step 1 below** — the graph's `simplify` node owns the `/simplify` pass, and running it here would double-simplify. Still run sub-steps 2 and 3: the graph's `simplify` node does not perform the comment audit (it operates on design-doc prose, not worker code) or re-run the tests, and the comment audit's strip-personal-workflow-artifacts mandate must run on every path or worker code risks shipping with internal tracking references intact. Legacy `krust implement` and oneshot runs (no such signal) run all three sub-steps.

After integration is green, tighten the code the workers produced before it leaves the skill. There is no PR yet, so everything here is scoped to the implementation's **working-tree changes** (the uncommitted diff).

1. **Simplify the code.** Invoke the `/simplify` skill over the changes. It applies reuse, simplification, efficiency, and altitude cleanups in place — collapsing duplication, swapping reinvented helpers for existing ones, and removing complexity the parallel workers couldn't see across task boundaries.

2. **Audit comments for load-bearing value.** Review every comment the implementation added or changed and cut the ones that don't earn their place:
   - **Always strip personal workflow artifacts** (no judgment call — these must never ship in code): beads task IDs (e.g. `jmcfarland-gzr4`), links or paths to design docs (`$COCKPIT_DIR/state/designs/…` or any cockpit reference), "see the plan / per the design" pointers, and any other internal tracking reference. They are meaningless to a repo reader and leak private context.
   - **Remove** comments that explain *what* well-named code already says, that reference the task/PR ("added for X"), or that are tombstones (`# renamed from…`, stale TODOs).
   - **Collapse** multi-paragraph docstrings or blocks to the one line that carries the load.
   - **Keep** only the non-obvious *why*: hidden constraints, invariants, ordering requirements, workaround-for-bug notes. When in doubt, keep — deleting load-bearing context is worse than leaving a marginal comment.

3. **Re-run the Step 6 tests.** Both passes edit real code, so re-run the same tests from Step 6. If a cleanup broke something, fix it or revert that specific change — never report with red tests.

Do not run any git commands — the changes stay in the working tree for krust.

### 8. Archive consumed design

If the design document lives in `$COCKPIT_DIR/state/designs/`, you'll archive it in Step 9. This runs in **every** mode, including graph mode — the graph does not own archiving (its `write_artifact` and `finish` nodes do not move the design to `finished/`). In legacy/oneshot mode the archive runs after `krust artifact implementations`, so the reason can use the resolved implementation slug (which may differ from the input slug if a collision was auto-bumped to `-v2`..`-v5`); in graph mode `krust artifact` runs in the graph's `write_artifact` node rather than the skill, so the archive reason falls back to the input `$slug`. If the design wasn't from the cockpit, no archive is needed.

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

Use the Write tool to save the report as a file at `$KRUST_OUT` (if set) or `/tmp/implement-report-<bd_id>.md`. You MUST write the file to disk — do not just output the report as conversation text.

Then run these commands via Bash. The `krust artifact` + `krust bd-finish` block is **skipped when the invoking prompt says the graph owns publication** (graph mode); there the graph's own `write_artifact` and `finish` nodes run them, so /implement must not. The **`krust archive designs` step always runs** (Step 8) — the graph does not archive, so /implement owns it in every mode. Legacy and oneshot runs (no graph signal) run the whole block as normal.

```bash
# Publication block — SKIP in graph mode (the graph's write_artifact + finish nodes own this).
# Emit the implementation artifact and capture the resolved slug. The resolved
# slug may differ from the input if there was a collision (auto-bumped to -v2..-v5).
result=$(krust artifact implementations "$slug" "$report_path")
final_slug=$(echo "$result" | jq -r .slug)

if [ "$final_slug" != "$slug" ]; then
  echo "Slug collided; resolved to $final_slug"
fi
# (end publication block)

# Archive the consumed design (Step 8) — ALWAYS runs, including graph mode.
# In legacy/oneshot mode use the resolved $final_slug so the reason matches the
# implementation-report filename; in graph mode $final_slug is unset (krust artifact
# ran in the graph node, not here), so fall back to the input $slug.
# Skip only if the design wasn't in $COCKPIT_DIR/state/designs/.
archive_slug="${final_slug:-$slug}"
krust archive designs "$design_file" "implemented: $archive_slug"

# Publication block (continued) — SKIP in graph mode.
krust bd-finish "$bd_id"
```

`krust artifact` declares the report as an output artifact and prints a JSON line on stdout with the resolved `slug` and `path`. `krust bd-finish` closes the orchestration task (no-op under krust — the wrapper closes it on approval).

## Rules

- **Do not start without a design** — if there's no design document, stop — unless the invoking prompt supplied a brief as the spec (see Input), in which case use that brief as the design source and proceed
- **Skills do not run git directly** — krust owns all git operations. Do not commit, branch, stash, merge, or run any git commands.
- **No worktrees** — do not use `isolation: "worktree"` on worker agents
- **Shared interfaces first** — create them before spawning workers to prevent drift
- **Workers follow the spec exactly** — no freelancing, no extra methods, no bonus abstractions
- **Match existing patterns** — look at neighboring code for conventions before writing new code
- **Tests are required** — every worker writes tests for its code
- **Report deviations** — if implementation must differ from design, explain why
- **## Tracking is mandatory** — output must include Beads task IDs
