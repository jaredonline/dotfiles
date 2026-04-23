You are shipping a completed implementation as a pull request. Your goal is to produce a PR title and body from the design and implementation report, then emit a `submit` action for krust to execute.

## Input

Inputs come from environment variables set by krust:

- `KRUST_BEADS_ID` — tracking task id
- `KRUST_OUT` — path to write the PR body markdown (artifact)
- `ACTIONS_DIR` — directory where this skill emits action JSON files
- `KRUST_SHIP_EXISTING_PR` — existing PR number if one exists; empty otherwise
- `KRUST_SHIP_USE_GRAPHITE` — `"1"` if krust preflight selected Graphite, else `"0"`

Krust writes the precomputed diff to `$ACTIONS_DIR/inputs/diff.patch` before invoking this skill.

Krust blocks `Bash(git:*)` via `disallowed_tools`. Do not attempt `git`, `gh`, or `gt` — krust owns all VCS operations.

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Read task metadata | No — main agent | Discover design + implementation artifacts |
| 2. Discover design doc | No — main agent | Needed for PR context |
| 3. Discover implementation report | No — main agent | Needed for PR changes summary |
| 4. Read precomputed diff | No — main agent | Krust already wrote it — skill must not re-run git |
| 5. Delegate to /pr for title+body | No — single sub-agent | Reuse /pr's voice and title conventions |
| 6. Write PR body artifact | No — main agent | Persist to `$KRUST_OUT` |
| 7. Emit submit action | No — main agent | One action JSON per invocation |

## Process

### 1. Read task metadata

```bash
bd show "$KRUST_BEADS_ID" --json
```

Extract:
- `metadata.krust.inputs.design_path` — design doc path (may be unset)
- `metadata.krust.artifact_path` — implementation report path (may be unset)

### 2. Discover the design doc

If `metadata.krust.inputs.design_path` is set, use it directly.

Otherwise, fall back to cockpit discovery (same loop as `/pr` step 2):

```bash
BRANCH_SLUG="${KRUST_BRANCH##*/}"   # or derive from the beads task title
grep -rl "$BRANCH_SLUG" \
  "$COCKPIT_DIR/state/designs/" \
  "$COCKPIT_DIR/state/designs/finished/" \
  2>/dev/null
```

Resolution: exactly one match — use it. Multiple — pick the most recently modified. None — proceed without a design doc (the PR will still ship).

### 3. Discover the implementation report

If `metadata.krust.artifact_path` is set, use it directly.

Otherwise, scan `$COCKPIT_DIR/state/implementations/` for a report referencing `$KRUST_BEADS_ID` or the branch slug. Pick the most recently modified match. If none, proceed without an implementation report.

### 4. Read the precomputed diff

```bash
cat "$ACTIONS_DIR/inputs/diff.patch"
```

Krust wrote this file during preflight. Do not run `git diff` — git is blocked, and the precomputed diff is authoritative.

### 5. Delegate title+body generation to /pr

Spawn `/pr` as a sub-agent (Agent tool, `model=opus`). Pass the discovered design doc path, the implementation report path, and the diff content. Instruct the sub-agent:

- Produce only the PR title and the PR body markdown — do not run git, gh, or gt.
- Follow `/pr`'s title convention (`[area] short description`, lowercase, under 70 chars).
- Follow `/pr`'s body template (`## Context`, `## Approach`, `## Reviewer guide`, `## Changes`, `## Testing`).
- Run the voice pass (`ghost-write`) before returning.
- Return the title on the first line, a blank line, then the full body markdown.

Parse the sub-agent's response into `title` and `body` strings.

### 6. Write the PR body artifact

Write `body` to `$KRUST_OUT`. This is the skill's artifact for krust.

### 7. Emit the submit action

Write exactly one action JSON to `$ACTIONS_DIR/<random>.json` (use `uuidgen`, `mktemp`, or a timestamp-based name):

```json
{
  "type": "submit",
  "target_repo": "",
  "branch": "",
  "title": "<from /pr>",
  "body": "<from /pr>",
  "draft": true,
  "use_graphite": false,
  "existing_pr_number": null,
  "commit_message": "<short, derived from title>"
}
```

Populate each field:

- `target_repo`, `branch` — empty strings. Krust overwrites both before `handle_submit` runs.
- `title`, `body` — verbatim from the `/pr` sub-agent output.
- `draft` — `true` when `KRUST_SHIP_EXISTING_PR` is empty (new branch); `false` when an existing PR is being updated. Krust may override based on CLI flags.
- `use_graphite` — `true` if `KRUST_SHIP_USE_GRAPHITE == "1"`, else `false`.
- `existing_pr_number` — `null` if `KRUST_SHIP_EXISTING_PR` is empty; otherwise the integer PR number.
- `commit_message` — derived from `title` by stripping bracketed area tags and lowercasing. Example: `[ship] add skill for krust` → `add skill for krust`.

Exit after writing the action. Krust reads `$ACTIONS_DIR`, overwrites `target_repo`/`branch`, and runs `handle_submit`.

## Rules

- **No git, gh, or gt** — krust blocks these tools and owns all VCS work.
- **Read the diff from `$ACTIONS_DIR/inputs/diff.patch`** — never invoke `git diff`.
- **Exactly one `submit` action per invocation** — do not emit multiple action files.
- **Write only to `$KRUST_OUT` and `$ACTIONS_DIR`** — no other filesystem writes.
- **Do not call `bd close`, `krust bd-finish`, or push anything** — krust owns task lifecycle and publishing.
- **Delegate to `/pr` for voice and formatting** — do not reimplement title or body rules here.
- **Terminate on completion** — no interactive loops; krust drives any follow-up iterations.
