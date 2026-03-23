You are decomposing a design into a Beads task graph. Your goal is to produce a set of claimable, self-contained tasks that agents can execute independently — solo or as a swarm.

## Input

The user provides a design doc path or reference (from `/design` output). If no design is available, stop and tell the user to run `/design` first.

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Create Beads task | No — main agent | Track work before starting |
| 2. Parse design | No — main agent | Read input |
| 3. Create epic | No — main agent | Container for tasks |
| 4. Decompose into tasks | No — main agent | Sequential creation with dep wiring |
| 5. Create integration task | No — main agent | Depends on knowing all task IDs |
| 6. Output | No — main agent | Format results |

## Process

### 1. Create Beads task

Run `bd create --title="Plan: [design name]" --type=task` and store the returned task ID. Claim it: `bd update <id> --claim`.

### 2. Parse the design

Read the design doc and extract:
- **Components** to build (from Architecture section)
- **Interfaces** to implement (from Interfaces section)
- **Data schemas** to create/modify (from Data Schemas section)
- **Dependencies** between components (from Data Flow section)
- **Invariants** that constrain implementation (from Invariants section)

### 3. Create the epic

```
bd create --title="Epic: [design name]" --type=epic \
  --description="Implementation of [design name]" \
  --context="Design doc: [path]"
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
  --context="Design: [design doc path]"
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

### 5. Create integration task

```
bd create \
  --title="Integration: [design name]" \
  --description="Merge all implementation branches. Run full test suite. Fix integration issues." \
  --type=task \
  --parent=<epic-id>
```

This task depends on ALL implementation tasks. Wire deps: `bd dep add <integration-id> <each-task-id>`.

### 6. Output

Close the planning task: `bd close <plan-task-id>`.

Present the plan:

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
- **Close the planning task** — before outputting the ## Tracking section
