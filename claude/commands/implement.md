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
| 7. Close tasks and report | No — main agent | Summarizes results with ## Tracking |

## Process

### 1. Create Beads task

Run `bd create --title="Implement: [design name]" --type=task` and store the returned task ID.
Claim it: `bd update <id> --claim`.

### 2. Detect mode

Check if a task graph already exists (from `/plan`):

```
bd list --parent=<epic-matching-design> --status=open
```

- **If results exist** → Mode A (task graph exists, proceed to Step 3A)
- **If empty** → Mode B (no task graph, proceed to Step 3B)

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

Create Beads tasks for each component: `bd create --title="..." --description="..." --type=task` with interface spec, file scope, and acceptance criteria in the description. Wire deps with `bd dep add`.

### 4. Create shared interfaces

Before spawning workers, create any shared types, interfaces, or proto definitions that multiple workers need. This prevents workers from inventing incompatible interfaces.

Write these shared files before spawning workers.

### 5. Spawn workers (parallel)

For each ready task (no unresolved deps), spawn a worker:

**Worker** (Agent, model=opus):
> You are implementing one task from a design.
>
> ## Your Task
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
> - Follow the interface spec exactly — do not add methods, fields, or parameters not in the spec
> - Match existing code patterns in the repo (error handling, naming, structure)
> - Write tests for the code you write
> - Do not modify files outside your task scope
> - Do not run any git commands (no commits, no branching, no stashing)
> - If you're blocked or find the design is ambiguous, report the issue — do not guess

Spawn ALL ready workers in ONE message.

For tasks with unresolved deps: wait for blocking workers to complete, then spawn the next wave.

### 6. Integrate

After all workers complete:

- Review the changes made by all workers for consistency
- Run any existing tests (`go test ./...`, `npm test`, `pytest`, etc.)
- Fix integration issues — these are usually import paths, type mismatches, or missing glue code
- Do not run any git commands (no commits, no branching)

If any check fails, fix it before reporting.

### 7. Close tasks and report

For each completed task: `bd close <task-id>`
Close the orchestration task: `bd close <orchestration-task-id>`

Report to the user:

```markdown
# Implementation Report

## Tasks Completed
[list task IDs and titles]

## Files Created/Modified
[list files]

## Test Results
[pass/fail summary]

## Deviations
[any differences from the design, with rationale]

## Tracking
- Beads: <orchestration-task-id> — closed
- [list all implementation task IDs and status]
```

## Rules

- **Do not start without a design** — if there's no design document, stop
- **No git operations** — do not commit, branch, stash, merge, or run any git commands. The user manages git themselves
- **No worktrees** — do not use `isolation: "worktree"` on worker agents
- **Shared interfaces first** — create them before spawning workers to prevent drift
- **Workers follow the spec exactly** — no freelancing, no extra methods, no bonus abstractions
- **Match existing patterns** — look at neighboring code for conventions before writing new code
- **Tests are required** — every worker writes tests for its code
- **Report deviations** — if implementation must differ from design, explain why
- **## Tracking is mandatory** — output must include Beads task IDs
