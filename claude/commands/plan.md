You are decomposing a design into a Beads task graph. Your goal is to produce a set of claimable, self-contained tasks that agents can execute independently — solo or as a swarm.

## Input

If `$KRUST_BEADS_ID` is set, inputs come from beads metadata (see Step 1). Otherwise, the user provides a design doc path or reference (from `/design` output). If no design is available, stop and tell the user to run `/design` first.

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

If `$KRUST_BEADS_ID` is set:
- The tracking task already exists — use `$KRUST_BEADS_ID` as the task ID.
- Read inputs from beads metadata: `bd show $KRUST_BEADS_ID --json`
- Extract `design_path` from `.metadata.krust.inputs.design_path`
- Do NOT create a planning tracking task — krust already created one.
- Skip project labeling entirely.

If `$KRUST_BEADS_ID` is not set:

**Project labeling**: Read `$COCKPIT_DIR/project-tree.json` (skip if missing or `COCKPIT_DIR` unset). Review the project list to understand the landscape of active projects and their labels. Determine which project this task belongs to by matching `cwd` against project `path` fields and matching the task topic against project names. If exactly one project matches, use its `labels` array. If ambiguous or no match, ask the user which project this is for. Store the resolved labels for all `bd create` calls in this skill invocation (epic, child tasks, integration task).

Run `bd create --title="Plan: [design name]" --type=task --labels=<resolved-labels>` and store the returned task ID. Claim it: `bd update <id> --claim`.

### 2. Parse the design

Read the design doc and extract:
- **Components** to build (from Architecture section)
- **Interfaces** to implement (from Interfaces section)
- **Data schemas** to create/modify (from Data Schemas section)
- **Dependencies** between components (from Data Flow section)
- **Invariants** that constrain implementation (from Invariants section)

### 3. Create the epic

If `$KRUST_BEADS_ID` is set:
- Do NOT create the epic via bd commands. Design the epic title and description, but defer creation to the `create_task_graph` action emitted in Step 6.

If `$KRUST_BEADS_ID` is not set:

```
bd create --title="Epic: [design name]" --type=epic \
  --description="Implementation of [design name]" \
  --context="Design doc: [path]" \
  --labels=<resolved-labels>,type:plan-epic \
  --metadata='{"krust":{"artifact_type":"plan-epic","project":"<project-id>","inputs":{"plan_epic":{"plan_task_id":"<plan-task-id>"}}}}'
```

Where `<plan-task-id>` is the id returned by `bd create` in Step 1 (the plan tracking task) and `<project-id>` is the resolved project id from `project-tree.json` (see Step 1's "Project labeling" paragraph). Appending `type:plan-epic` to the labels and the `--metadata` flag identifies this epic as a plan-epic for downstream pipeline discovery.

Store the epic ID. All tasks will be children of this epic.

### 4. Decompose into claimable tasks

If `$KRUST_BEADS_ID` is set:
- Do NOT create tasks via bd commands. Design the full task list with titles, descriptions, and dependency refs, but defer creation to the `create_task_graph` action emitted in Step 6.
- Assign each task a short ref label (e.g., "A", "B", "C") for use in `depends_on` arrays.

If `$KRUST_BEADS_ID` is not set, for each component, create a Beads task:

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

### 5. Create integration task

If `$KRUST_BEADS_ID` is set:
- Do NOT create the integration task via bd commands. Include it as the final task in the `create_task_graph` action (Step 6) with `depends_on` listing all other task refs.

If `$KRUST_BEADS_ID` is not set:

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

If `$KRUST_BEADS_ID` is set:

Write plan summary to `$KRUST_OUT` with YAML frontmatter:

```markdown
---
beads_id: <$KRUST_BEADS_ID>
design: <design-path>
---
# Plan: [Design Name]

## Epic
- Design: [path to design doc]

## Tasks
| Ref | Title | Depends On | File Scope |
|-----|-------|-----------|------------|
| A   | ...   | —         | ...        |
| B   | ...   | A         | ...        |

## Execution
- **Solo**: Run `/implement` — it will read this task graph and orchestrate workers
- **Multi-agent**: Each agent runs `bd ready --parent=<epic-id>`, claims a task, implements it in a worktree
```

Emit `create_task_graph` action to `$ACTIONS_DIR/task-graph.json`:

```json
{
  "type": "create_task_graph",
  "parent": "<KRUST_BEADS_ID>",
  "project": "<value from `bd show $KRUST_BEADS_ID --json`.metadata.krust.project>",
  "labels": ["<plan task labels stripped of 'type:plans'>"],
  "plan_task_id": "<KRUST_BEADS_ID>",
  "epic": {
    "title": "Implement: Feature X",
    "description": "Epic description with context"
  },
  "tasks": [
    {
      "ref": "A",
      "title": "...",
      "description": "Full self-contained description with interface specs pasted in",
      "depends_on": []
    },
    {
      "ref": "B",
      "title": "...",
      "description": "Full self-contained description with interface specs pasted in",
      "depends_on": ["A"]
    }
  ]
}
```

Populate the top-level fields from the plan tracking task via `bd show $KRUST_BEADS_ID --json`:
- `parent`: `$KRUST_BEADS_ID` (the plan tracking task becomes the parent of the epic).
- `project`: read from `.metadata.krust.project`.
- `labels`: the plan task's `labels` array, with any `type:plans` entry stripped out so the epic and child tasks inherit project labels without the plan-task marker.
- `plan_task_id`: `$KRUST_BEADS_ID`.

Fallback: if any of `project`, `labels`, or `plan_task_id` cannot be resolved (e.g. `project` missing from the plan task metadata), omit all three fields — the handler falls back to legacy behavior. Do NOT hardcode project names.

The `tasks` array must be dependency-ordered — every ref in a task's `depends_on` must appear earlier in the array. Task descriptions must be self-contained (specs pasted in, not referenced).

If the design doc came from an exploration or design file, emit archive action to `$ACTIONS_DIR/archive-design.json`:

```json
{
  "type": "archive",
  "kind": "designs",
  "file": "<design-filename>",
  "reason": "finished: plans-<slug>"
}
```

Mark skill complete: `bd update $KRUST_BEADS_ID --set-metadata='skill_complete=true'`

Do NOT:
- Run `bd close` on the krust tracking task
- Run git operations
- Create beads tasks directly

If `$KRUST_BEADS_ID` is not set:

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
- **Close the planning task** — before outputting the ## Tracking section (standalone only; under krust, set `skill_complete` metadata instead)
