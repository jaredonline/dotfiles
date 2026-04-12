You are recovering session context after a /clear, compaction, or new session. Your goal is to reload beads workflow state, active work, and recent history so you can resume work immediately.

## Process

### 1. Reload beads context

Run `bd prime` to restore workflow rules and command reference. Then append persistent memories:

```bash
bd prime
bd prime --export | sed -n '/^## Persistent Memories/,$p'
```

The first command outputs the PRIME.md override (custom workflow context). The second extracts the dynamic memories section from the default output and appends it.

If `bd` is not available (command not found), skip to Step 4 and note that beads is unavailable.

### 2. Show active work

Run `bd list --status=in_progress` to see currently claimed tasks.

### 3. Resolve ancestor memory

Read `$COCKPIT_DIR/project-tree.json` (skip if file missing or COCKPIT_DIR unset).

Use inline Python to find ancestor projects for the current working directory:

```bash
python3 ~/.claude/scripts/ancestor-memory.py
```

For each ancestor found, read the MEMORY.md file and output:

```
## Inherited context from: <project name>
<contents of MEMORY.md>
```

Skip gracefully if:
- COCKPIT_DIR is unset and ~/ai-cockpit/project-tree.yml doesn't exist
- cwd is not found in the project tree
- Python is not available

### 4. Load recent daily summary

Find and read the most recent daily summary:

```bash
LATEST=$(~/.claude/scripts/latest-summary.sh)
```

If a file exists, read it with the Read tool. This contains PRs, action items, stack status, and review activity.

If no file exists or `$COCKPIT_DIR` is unset, skip this step.

### 5. Output status

Print a brief summary:

```
Session primed. N tasks in progress. Daily summary from YYYY-MM-DD loaded.
```

Adjust the message based on what was available:
- If no tasks in progress: "Session primed. No active tasks."
- If no daily summary found: omit the daily summary mention
- If beads unavailable: "Session primed (beads unavailable). Read CLAUDE.md for workflow context."
- If ancestor context loaded: append "Inherited context from N ancestor(s)."

## Rules

- No subagents — run everything synchronously in the main agent
- Do not load skill file contents — skills are invoked on-demand via /name
- Do not duplicate what CLAUDE.md already provides (it's auto-loaded)
- Idempotent — safe to run multiple times with no side effects
