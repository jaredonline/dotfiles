You are decomposing a design into a Beads task graph. Your goal is to produce a set of claimable, self-contained tasks that agents can execute independently — solo or as a swarm.

## Input

If `$KRUST_BEADS_ID` is set, inputs come from beads metadata (see Step 1). Otherwise, the user provides a design doc path or reference (from `/design` output). If no design is available, stop and tell the user to run `/design` first.

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Create Beads task | No — main agent | Track work before starting |
| 2. Parse design & pick mode | No — main agent | Read input; decide single vs stacked |
| 3. Create epic(s) | No — main agent | One epic in single mode; one per layer in stack mode |
| 4. Decompose into tasks | No — main agent | Sequential creation with dep wiring |
| 5. Create integration task(s) | No — main agent | Depends on knowing all task IDs |
| 5.5. Self-check (stack mode only) | No — main agent | Reject bad layer splits before emission |
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

### 2. Parse the design & pick mode

Read the design doc and extract:
- **Components** to build (from Architecture section)
- **Interfaces** to implement (from Interfaces section)
- **Data schemas** to create/modify (from Data Schemas section)
- **Dependencies** between components (from Data Flow section)
- **Invariants** that constrain implementation (from Invariants section)
- **Decomposition Hints** if present (the optional `## Decomposition Hints` section emitted by `/design`)

**Trust file paths as canonical.** Paths referenced in the design (e.g. `krust/src/downstream.rs:465`) point to real files — open them with the Read tool, don't search for them. If you must locate an unfamiliar file, use `rg --files` or `ls` scoped to CWD. Never run `find /`, `find $HOME`, or any search whose root is broader than the project.

#### Mode selection

Stack mode is gated behind the `KRUST_STACK_ENABLED` env var as a rollout kill switch. If `KRUST_STACK_ENABLED` is not set to `1`, choose **single mode** unconditionally — skip the checks below and ignore any `## Decomposition Hints` content. This gate exists so stack mode can be disabled in production without code changes if it misbehaves.

When `KRUST_STACK_ENABLED=1`, choose **stack mode** when ALL of the following hold:

1. The design contains a `## Decomposition Hints` section with **>= 2** numbered entries.
2. Estimated total diff across the design is **> 200 LOC** (sum of touched files and interface specs as a rough proxy — if files-to-modify counts and interface bodies suggest a small change, single mode).
3. The design's `## Data Flow` section describes **>= 2 logical layers** (distinct sequential steps or boundary crossings).

Otherwise, choose **single mode** (today's behavior, byte-equivalent modulo a new `mode: single` frontmatter line).

If stack mode is selected:
- Derive the **stack slug** from the design filename (e.g. `state/designs/design-stacked-pr-workflow.md` → `stacked-pr-workflow`).
- Parse each `## Decomposition Hints` entry into a layer: `position` (1-indexed), `layer_slug` (kebab-case of the layer name), `description`, `merge_alone_invariant`, `touches`.
- Compute branch names: `<stack-slug>-<NN>-<layer-slug>` where `NN` is zero-padded (`01`, `02`, ...).
- Layer 1's base branch is always `main` (non-main bases are out of scope for v1).
- For layer N >= 2, the base branch is layer (N-1)'s branch name.
- Resolve `vcs_strategy` from this lookup order: (1) `$KRUST_SHIP_VCS_STRATEGY` if set, (2) the project's krust config if present, (3) default `git`. The chosen string goes into the plan frontmatter verbatim.

Record the chosen mode and (if stacked) the layer list for use in steps 3–6.

### 3. Create the epic(s)

**Stack mode**: design one epic per layer. Each layer's epic title is `Layer <NN>: <layer-name>`; its description summarizes that layer's scope and merge-alone invariant. Defer creation to Step 6 — under `$KRUST_BEADS_ID`, layer epics are created by the per-layer `create_task_graph` actions emitted in Step 6. Outside krust (no `$KRUST_BEADS_ID`), stack mode is not supported in v1; fall back to single mode and log the reason.

**Single mode** continues below.

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

**Stack mode**: decompose each layer independently. Every task belongs to exactly one layer — no task spans layers. Task refs are scoped per layer (each layer restarts at `A`, `B`, `C`). `depends_on` refs only point to tasks within the same layer; cross-layer ordering is expressed via the layer's epic `blocked_by` (see Step 6).

The rules below apply per-layer in stack mode and to the whole plan in single mode.

If `$KRUST_BEADS_ID` is set:
- Do NOT create tasks via bd commands. Design the full task list with titles, descriptions, and dependency refs, but defer creation to the `create_task_graph` action(s) emitted in Step 6.
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

### 5. Create integration task(s)

**Stack mode**: each layer gets its own integration task as the final entry in that layer's `tasks` array, depending on every other task ref within the same layer. There is no global integration task — layer-N+1's epic `blocked_by` layer-N's epic provides the cross-layer ordering.

**Single mode** continues below.

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

### 5.5. Self-check (stack mode only)

Skip this step in single mode.

For each proposed layer L in order, verify the split:

- **Merge-alone test** — if L were the last layer to ever land (all later layers cancelled), does the codebase still compile, do tests still pass, and is there no dead code introduced by L? Use the layer's `Merge-alone invariant` and `Touches` lines plus the design's Architecture and Interfaces sections to reason about this. If a layer adds a type, function, or config field whose only users live in later layers, the layer fails this check.
- **Type-incomplete check** — does L introduce types, traits, methods, or fields that have no in-layer caller and would be dead code without later layers? If yes, L fails.
- **Circular dependency check** — does any layer's `Merge-alone invariant` depend on behavior introduced by a strictly later layer? If yes, the split is circular.

If every layer passes all three checks, proceed to Step 6 with stack mode.

If any layer fails: **fall back to single mode** and record the reason. In Step 6's output, the plan markdown MUST include a `## Mode: Single` heading with `**Reason**: stack rejected because <which layer failed which check>` underneath it (see the single-mode template). Re-derive the single-mode task list as if no decomposition hints existed.

### 6. Output

If `$KRUST_BEADS_ID` is set, follow the **Single mode** or **Stack mode** branch below based on Step 2's selection.

#### Single mode

Write plan summary to `$KRUST_OUT` with YAML frontmatter:

```markdown
---
beads_id: <$KRUST_BEADS_ID>
design_path: <design-path>
mode: single
---
# Plan: [Design Name]

## Mode: Single
**Reason**: <one sentence — only present when single mode is reached via Step 5.5 stack-rejection fallback; omit this heading when single mode was chosen directly>

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

The only differences from prior single-mode output are the new `mode: single` line in the frontmatter and the optional `## Mode: Single` heading (only emitted on Step 5.5 stack-rejection fallback); everything else (headings, tables, body text, action JSON shape) is byte-equivalent.

Emit one `create_task_graph` action to `$ACTIONS_DIR/task-graph.json`:

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

#### Stack mode

**Replan detection (run before writing the markdown):**

Replanning a stack must preserve merged layers' status and PR numbers — a re-run of `/plan` after layer 1 ships must not reset its row back to `pending`. Before writing the new plan, read any existing plan markdown at the target path (`$KRUST_OUT`; if unset, the design-derived path you would otherwise write to) and merge incrementally:

1. **Read the existing plan**: If a file exists at the target path, read it. If no file exists, or the file has no `## Stack Status` table, treat this as a fresh plan — every new row starts as `pending`/`—` and skip the rest of this step.
2. **Parse the existing Stack Status table**: For each row, extract `position` (the `#` column), `layer_slug` (the `Layer` column), `branch`, `base`, `status`, and `PR`. Build a lookup keyed by `(position, layer_slug)` — matching on both guards against layer renames or re-ordering between runs.
3. **Merge per layer**: For each layer you are about to emit:
   - Look it up in the parsed table by `(position, layer_slug)`.
   - If a match exists AND its `status` is not `pending` (e.g. `merged`, `open`, `in-review`, `closed`, anything non-pending), **preserve the existing `status` and `PR` verbatim** in the new row.
   - Otherwise (no match, or matched row is still `pending`), use `status = pending` and `PR = —`.
4. **New layers added by replanning** (layer not present in the old table) are written as `pending`/`—`.
5. **Removed layers** (present in old table but not in new plan) are dropped — do not carry them forward.

The `Branch`, `Base`, and `Epic` columns come from the freshly computed values in Step 2 / Step 6, not from the old table — only `Status` and `PR` are preserved across replans. Record any preserved rows in a short note printed to the user after writing so they can confirm the merge was intentional.

Write plan summary to `$KRUST_OUT` with stacked frontmatter and one layer section per decomposition hint:

```markdown
---
beads_id: <$KRUST_BEADS_ID>
design_path: state/designs/design-<slug>.md
mode: stacked
vcs_strategy: graphite   # or "git"; only present when mode=stacked
---

# Plan: <name>

## Mode: Stacked (N layers)
**Reason**: <one sentence — e.g. "Design's data flow decomposes into 3 independent migration steps">

## Terminal State
<What the codebase looks like after ALL layers land. /implement reads this from any layer's invocation, so it must describe the full end state — not just one layer's slice.>

## Stack Status
| # | Layer | Branch | Base | Status | PR | Epic |
|---|-------|--------|------|--------|-----|------|
| 1 | add-stack-manifest | stacked-pr-workflow-01-add-stack-manifest | main | pending | — | <epic-id> |
| 2 | route-design-through-stack | stacked-pr-workflow-02-route-design | stacked-pr-workflow-01-add-stack-manifest | pending | — | <epic-id> |

## Layers

### Layer 1: add-stack-manifest
**Beads epic**: <epic-id>
**Merge-alone invariant**: <property that must hold if this PR ships alone>

#### Tasks
| Ref | Title | Files | Depends on |
|-----|-------|-------|------------|
| A   | ...   | ...   | —          |

#### Interface specs
<pasted-in specs for layer 1 — same paste-in detail as today's single-mode plan format>

### Layer 2: route-design-through-stack
**Beads epic**: <epic-id>
**Merge-alone invariant**: ...

#### Tasks
| Ref | Title | Files | Depends on |
|-----|-------|-------|------------|
| A   | ...   | ...   | —          |

#### Interface specs
<pasted-in specs for layer 2>
```

Notes on the stacked template:
- On a fresh plan, all Stack Status rows start with `Status = pending`, `PR = —`, and `Epic = <epic-id>` as a literal placeholder token. On a replan (see "Replan detection" above), rows for already-progressed layers carry their prior `Status` and `PR` forward verbatim while `Epic` is re-resolved. Once krust processes the per-layer `create_task_graph` actions and assigns epic IDs, the placeholders are resolved by krust (e.g. `krust stack <slug> --update`) — this skill writes them as placeholders and does not rewrite the artifact after action processing.
- The `vcs_strategy` frontmatter value comes from Step 2's resolution (env > project config > `git` default). It is the only place this skill mentions VCS at all — and only as a data field, not a command.
- Each layer's `Interface specs` subsection paste-in is independent: a layer should be implementable from its own subsection without scrolling to other layers, but `## Terminal State` provides the cross-layer end-goal view.
- **Operator action — manual `merged` flip**: krust automatically flips a layer to `in_review` when its PR opens, but there is no automated watcher that advances `in_review` → `merged`. After a layer's PR lands on `main`, the operator MUST run `krust stack <stack-slug> --update <position> merged` by hand. Skipping this leaves the Stack Status table showing `in_review` indefinitely and causes `krust stack <stack-slug> --abandon` to flip the row to `abandoned` instead of preserving it as `merged`. The stack itself does not stall (the next layer advances on `in_review`), but the historical record is wrong until the operator runs the update. Surface this in the plan body if helpful for the team — the skill itself only writes the table.

Then emit one `create_task_graph` action per layer to `$ACTIONS_DIR/task-graph-<NN>.json` (NN zero-padded to match the branch). Each action carries an additional `stack_layer` block:

```json
{
  "type": "create_task_graph",
  "parent": "<KRUST_BEADS_ID>",
  "project": "<from plan task metadata>",
  "labels": ["<plan labels stripped of 'type:plans'>"],
  "plan_task_id": "<KRUST_BEADS_ID>",
  "stack_layer": {
    "stack_slug": "<stack-slug>",
    "position": <n>,
    "layer_slug": "<layer-slug>",
    "branch_name": "<stack-slug>-<NN>-<layer-slug>",
    "base_branch": "main",
    "invariant": "<text>"
  },
  "epic": {
    "title": "Layer <NN>: <layer-name>",
    "description": "<layer description with scope + invariant>",
    "blocked_by": "<layer-N-minus-1-layer-slug>"
  },
  "tasks": [
    {
      "ref": "A",
      "title": "...",
      "description": "Self-contained with interface specs pasted in",
      "depends_on": []
    }
  ]
}
```

Per-layer rules:
- `stack_layer.position` is 1-indexed.
- `stack_layer.base_branch` is `"main"` for position 1; for position N >= 2 it is the prior layer's branch name (`<stack-slug>-<NN-1>-<prev-layer-slug>`).
- The `epic` block for layer N >= 2 also carries `"blocked_by": "<layer-N-minus-1-layer-slug>"` (the prior layer's `layer_slug`, NOT an epic id — the skill never observes epic ids). Krust resolves the slug against the prior sibling action's freshly-assigned epic id and wires the dependency via `bd dep add`. Layer 1 has no `blocked_by`.

**Sequencing the actions** (krust processes them in filename order):

Write all `task-graph-<NN>.json` files up front — krust sorts the actions dir by filename, so layer-(N-1)'s action is always dispatched before layer-N's, and the in-process `layer_slug → epic_id` map is populated by the time layer-N's `blocked_by` is resolved.

1. Write the plan markdown artifact to `$KRUST_OUT` first (with `<epic-id>` placeholders).
2. Emit `task-graph-01.json` for layer 1 (no `blocked_by`).
3. For each subsequent layer N: emit `task-graph-<NN>.json` with `epic.blocked_by` set to layer (N-1)'s `layer_slug`.

The `tasks` array within each per-layer action must still be dependency-ordered (every ref in `depends_on` appears earlier in the same layer's `tasks` array). Task descriptions remain self-contained (specs pasted in, not referenced).

#### Both modes

If the design doc came from an exploration or design file, emit archive action to `$ACTIONS_DIR/archive-design.json`:

```json
{
  "type": "archive",
  "kind": "designs",
  "file": "<design-filename>",
  "reason": "finished: <slug>"
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
