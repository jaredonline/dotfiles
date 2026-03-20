You are creating a pull request for the current branch. Your goal is to produce a well-structured PR with a clear description that helps reviewers understand the changes.

## Process

### 1. Gather context

Run these in parallel:
- `git status` — check for uncommitted changes
- `git log main...HEAD --oneline` — all commits on this branch
- `git diff main...HEAD --stat` — files changed summary
- `git diff main...HEAD` — full diff

If there are uncommitted changes, ask the user if they want to commit first.

### 2. Check for design document

Look for a design document associated with this work:
- In conversation context
- Referenced in commit messages

If found, use it to inform the PR description.

### 3. Analyze changes

From the diff and commit history, identify:
- **Areas** — which services, systems, or components are touched (e.g. acme, sinatra, ai assistant, healthcheck, config mirror)
- **What changed** — new features, bug fixes, refactors, tests
- **Why it changed** — the motivation (from commits, design doc, or conversation)
- **What to watch** — areas reviewers should focus on
- **What's NOT included** — deliberate exclusions or follow-up work

### 4. Create the PR

#### Title format

Titles MUST start with area tags in brackets, followed by a lowercase description:

```
[area1] [area2] short description of the change
```

Examples:
- `[acme] [sinatra] use the socket instead of TCP`
- `[ai assistant] fix network call regression in entry point`
- `[ld relay] [healthcheck] fix the flag latency check`
- `[config mirror] [client_platform] update config mirror flags based on 2024 values`

Rules for area tags:
- Derive areas from the services, systems, or components touched in the diff
- Use lowercase, spaces allowed inside brackets
- One tag per area — use multiple tags if the change spans areas
- The description after the tags is lowercase and concise

```bash
gh pr create --title "[area] short description" --body "$(cat <<'EOF'
## Summary
<1-3 bullet points: what and why>

## Changes
<grouped list of what changed, by area>

## Design
<link to design doc if one exists, or brief rationale>

## Testing
<what was tested, how to verify>

## Rollback
<is this safely reversible? any special rollback steps?>

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

### 5. Report

Show the user the PR URL.

## Rules

- **Title under 70 characters** — use the description for details
- **Lead with the why** — reviewers care about motivation before mechanics
- **Don't commit for the user** — if there are uncommitted changes, ask first
- **Link the design doc** — if one exists, reference it in the PR body
- **Keep it concise** — a PR description is a summary, not documentation
