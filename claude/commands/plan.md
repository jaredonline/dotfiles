You are decomposing a design into a Beads task graph. Your goal is to produce a set of claimable, self-contained tasks that agents can execute independently — solo or as a swarm.

## Input

The design doc path or reference comes from `$ARGUMENTS` (typically `/design` output). If no design is available, stop and tell the user to run `/design` first.

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Create Beads task | No — main agent | Track work before starting |
| 2. Parse design | No — main agent | Read input |
| 3. Create epic | No — main agent | One epic for the plan |
| 4. Decompose into tasks | No — main agent | Sequential creation with dep wiring |
| 5. Create integration task | No — main agent | Depends on knowing all task IDs |
| 6. Output | No — main agent | Format results |

## Process

### 1. Create Beads task

**Project labeling**: Read `$COCKPIT_DIR/project-tree.json` (skip if missing or `COCKPIT_DIR` unset). Review the project list to understand the landscape of active projects and their labels. Determine which project this task belongs to by matching `cwd` against project `path` fields and matching the task topic against project names. If exactly one project matches, use its `labels` array. If ambiguous or no match, ask the user which project this is for. Store the resolved labels for all `bd create` calls in this skill invocation (epic, child tasks, integration task).

Run `bd create --title="Plan: [design name]" --type=task --labels=<resolved-labels>` and store the returned task ID. Claim it: `bd update <id> --claim`.

### 2. Parse the design

Read the design doc and extract:
- **Components** to build (from Architecture section)
- **Interfaces** to implement (from Interfaces section)
- **Data schemas** to create/modify (from Data Schemas section)
- **Dependencies** between components (from Data Flow section)
- **Invariants** that constrain implementation (from Invariants section)

**Trust file paths as canonical.** Paths referenced in the design (e.g. `src/downstream.rs:465`) point to real files — open them with the Read tool, don't search for them. If you must locate an unfamiliar file, use `rg --files` or `ls` scoped to CWD. Never run `find /`, `find $HOME`, or any search whose root is broader than the project.

### 3. Create the epic

```
bd create --title="Epic: [design name]" --type=epic \
  --description="Implementation of [design name]" \
  --context="Design doc: [path]" \
  --labels=<resolved-labels>
```

Store the epic ID. All tasks will be children of this epic.

### 4. Decompose into claimable tasks

For each component, create a Beads task:

```
bd create \
  --title="Implement: [component name]" \
  --description="[interface spec, file scope, acceptance criteria]" \
  --type=task \
  --parent=<epic-id> \
  --context="Design: [design doc path]" \
  --labels=<resolved-labels>
```

Each task description MUST include:
- **Interface spec**: Exact method signatures from the design (pasted in, not referenced)
- **File scope**: Which files to create or modify
- **Acceptance criteria**: How to verify the task is done

Wire dependencies: `bd dep add <task-id> <depends-on-id>`

Rules for task decomposition:
- No two concurrent tasks modify the same file. If unavoidable, serialize via deps.
- If a shared interface file is needed, the first task that creates it blocks all consumers via deps.
- Each task should be completable by one agent in one session.
- Task descriptions are self-contained — an agent can implement from the description alone.

### 5. Create the integration task

```
bd create \
  --title="Integration: [design name]" \
  --description="Merge all implementation branches. Run full test suite. Fix integration issues." \
  --type=task \
  --parent=<epic-id> \
  --labels=<resolved-labels>
```

This task depends on ALL implementation tasks. Wire deps: `bd dep add <integration-id> <each-task-id>`.

### 6. Output

Write the plan summary to `$COCKPIT_DIR/state/plans/plans-<slug>.md` (derive `<slug>` from the design filename), then `git add`/`commit`/`push` it directly:

```markdown
---
design_path: <design-path>
epic_id: <epic-id>
---
# Plan: [Design Name]

## Epic
- Beads: <epic-id>
- Design: [path to design doc]

## Tasks
| ID | Title | Depends On | File Scope |
|----|-------|-----------|------------|
| ... | ... | ... | ... |

## Execution
- **Solo**: Run `/implement` — it will read this task graph and orchestrate workers
- **Multi-agent**: Each agent runs `bd ready --parent=<epic-id>`, claims a task, implements it in a worktree
- **Coder swarm**: `bd dolt push` to share the graph, agents pull and claim, merge via PRs
```

The `Tasks` table must be dependency-ordered — every ID in a task's `Depends On` column must appear earlier in the table. Task descriptions (in beads) must be self-contained (specs pasted in, not referenced).

Close the planning task: `bd close <plan-task-id>`.

Present the plan to the user, including a `## Tracking` section noting the closed planning task:

```markdown
# Plan: [Design Name]

## Epic
- Beads: <epic-id>
- Design: [path to design doc]

## Tasks
| ID | Title | Depends On | File Scope |
|----|-------|-----------|------------|
| ... | ... | ... | ... |

## Execution
- **Solo**: Run `/implement` — it will read this task graph and orchestrate workers
- **Multi-agent**: Each agent runs `bd ready --parent=<epic-id>`, claims a task, implements it in a worktree
- **Coder swarm**: `bd dolt push` to share the graph, agents pull and claim, merge via PRs

## Tracking
- Beads: <plan-task-id> — closed
```

## Rules

- **Single agent, no subagents** — this is analytical work, not parallelizable
- **Task descriptions are self-contained** — paste interface specs into each task, don't reference
- **No file scope overlap** — concurrent tasks must not touch the same files
- **Serialize conflicts via deps** — if overlap is unavoidable, add a dependency
- **Integration task always last** — depends on all implementation tasks
- **Close the planning task** — before outputting the `## Tracking` section
