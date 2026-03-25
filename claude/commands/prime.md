You are recovering session context after a /clear, compaction, or new session. Your goal is to reload beads workflow state, active work, and recent history so you can resume work immediately.

## Process

### 1. Reload beads context

Run `bd prime` to restore workflow rules, command reference, and persistent memories.

If `bd` is not available (command not found), skip to Step 3 and note that beads is unavailable.

### 2. Show active work

Run `bd list --status=in_progress` to see currently claimed tasks.

### 3. Load recent daily summary

Find and read the most recent daily summary:

```bash
LATEST=$(ls -1 "$COCKPIT_DIR/state/news/"*.md 2>/dev/null | tail -1)
```

If a file exists, read it with the Read tool. This contains PRs, action items, stack status, and review activity.

If no file exists or `$COCKPIT_DIR` is unset, skip this step.

### 4. Output status

Print a brief summary:

```
Session primed. N tasks in progress. Daily summary from YYYY-MM-DD loaded.
```

Adjust the message based on what was available:
- If no tasks in progress: "Session primed. No active tasks."
- If no daily summary found: omit the daily summary mention
- If beads unavailable: "Session primed (beads unavailable). Read CLAUDE.md for workflow context."

## Rules

- No subagents — run everything synchronously in the main agent
- Do not load skill file contents — skills are invoked on-demand via /name
- Do not duplicate what CLAUDE.md already provides (it's auto-loaded)
- Idempotent — safe to run multiple times with no side effects
