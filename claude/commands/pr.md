You are creating a pull request for the current branch. Your goal is to produce a well-structured PR with a clear description that helps reviewers understand the changes.

## Process

### 0. Detect Graphite

Check once at the start:

```bash
command -v gt &>/dev/null
```

If `gt` is found, use Graphite commands in Step 4. If not, use `gh`/`git` throughout. This is a single detection point — do not re-check later.

If `gt submit` later fails with a configuration or initialization error, fall back to `gh pr create`. For other errors (auth, network), surface the error to the user.

### 1. Gather context and handle working tree

Run these in parallel:

```bash
git status
git symbolic-ref --short HEAD         # current branch name
git log main...HEAD --oneline         # commits on this branch (empty if on main)
git diff main...HEAD --stat           # files changed summary (empty if on main)
git diff main...HEAD                  # full diff (empty if on main)
git diff --name-only                  # unstaged files
git diff --cached --name-only         # staged but uncommitted files
```

Then apply this decision tree:

**If there are unstaged changes:**
Show the unstaged file list to the user and ask: "These files have unstaged changes. Include them in the PR?" Remember the answer for the next step.

**If on main:**
- If no staged changes AND no unstaged changes: stop with "Nothing to PR — you're on main with no changes."
- If no staged changes AND user excluded unstaged: stop with "No staged changes and you excluded unstaged ones. Stage what you want and rerun."
- Otherwise, create a branch and commit:
  - With Graphite: `gt create -m "<short summary>" {-a if including unstaged}`
  - Without Graphite: `{git add -A if including unstaged} && git checkout -b <branch-name> && git commit -m "<short summary>"`
- Remember that this PR was created from main (controls draft mode in Step 4).

**If NOT on main, but there are uncommitted changes (staged or newly staged):**
- With Graphite: `gt modify {-a if including unstaged}` — amends the current commit.
- Without Graphite: `{git add -A if including unstaged} && git commit -m "<short summary>"` — creates a new commit (never amend without Graphite).

**If NOT on main and everything is committed:** proceed directly to Step 2.

**Commit message generation:** When committing in this step, use a short `[area] description` message derived from the changed files. This is a lightweight pass — the full analysis happens in Step 3.

### 2. Check for design document

Look for a design document in this priority order. Stop at the first match.

**1. User-supplied path:** If the user provided a design doc path as an argument to /pr or mentioned one in conversation, read it directly.

**2. Cockpit discovery:** If `$COCKPIT_DIR` is set and `$COCKPIT_DIR/state/designs/` exists, search for a matching design doc:

```bash
BRANCH=$(git symbolic-ref --short HEAD)          # already gathered in Step 1
SLUG=${BRANCH##*/}                                # strip prefix (e.g., "jared/" → "fix-auth-flow")

grep -rl "$SLUG" "$COCKPIT_DIR/state/designs/" "$COCKPIT_DIR/state/designs/finished/" 2>/dev/null
```

Resolution: if exactly one match, use it. If multiple, use the most recently modified (filesystem mtime). If none, fall through.

**3. Conversation context:** Design doc content already present in the conversation.

**4. Commit messages:** References to design docs in commit messages.

If a design doc is found, read the full document and use its content to enrich the PR — its Problem section informs Context, Key Decisions inform Approach, Invariants inform Reviewer guide. Use what's relevant holistically; don't parse headings mechanically.

If cockpit discovery fails for any reason (missing dir, grep errors, unset env var), fall through silently — no errors, no warnings.

### 3. Analyze changes

From the diff and commit history, identify:
- **Problem**: What problem does this change solve? Why does it matter? (from design doc, commit messages, or conversation)
- **Solution rationale**: Why this approach? What alternatives were considered? What's deliberately out of scope?
- **Areas**: Which services/systems/components are touched
- **What changed**: New features, bug fixes, refactors, tests
- **Reviewer focus**: Non-obvious decisions, risky areas, edge cases to verify

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

#### PR body template

```markdown
## Context
<2-3 sentences: what problem this solves, why it matters, what triggered this work.
Not a list of file changes — the problem from the user/system perspective.>

## Approach
<Why this solution over alternatives. If there were meaningful alternatives
considered, note why they were rejected. If anything is deliberately out of
scope, say so here. 1-2 paragraphs max.
If a design doc was found, add: "Design: `<filename>`" at the end of this section.>

## Reviewer guide
<Where should reviewers focus their attention? What's non-obvious?
Flag: risky areas, new patterns introduced, assumptions made, edge cases to verify.
Help the reviewer skip straight to verifying correctness in the areas that matter.>

## Changes
<Grouped list of what changed, by area. Scannable bullet list — the mechanical "what".
The Context and Approach sections above provide the "why".>

## Testing
<What was tested, how to verify. Include specific test commands or scenarios.>

🤖 Generated with [Claude Code](https://claude.com/claude-code)
```

**Guidance for writing the PR body:**

The PR description should make the reviewer's job easier by front-loading answers to the questions they'll ask:
1. **Is this the right problem?** → Context section answers this
2. **Is this the right solution?** → Approach section answers this
3. **Where should I focus?** → Reviewer guide answers this
4. **What actually changed?** → Changes section answers this

Write Context and Approach as narrative prose, not bullet points. The reviewer needs to understand motivation before mechanics. Changes stays as a scannable list because reviewers ctrl-F it for specific areas.

#### Voice pass

After composing the PR body and before submitting, spawn a sub-agent (Agent, model=opus) to rewrite the prose in your voice. The sub-agent's prompt is the full content of `claude/commands/ghost-write.md`, with the composed PR body as the input text. Append these constraints to the prompt:

```
Additional constraints for this rewrite:
- Only rewrite the prose in ## Context, ## Approach, and ## Reviewer guide
- Return the FULL PR body with ## Changes and ## Testing untouched
- Preserve all ## headings exactly as-is
- Preserve inline references and formatting (links, filenames, code spans)
```

Use the sub-agent's returned text as the PR body for submission.

If the sub-agent fails (e.g., writing-voice memory not found), submit the original body and note in the Step 5 report: "Voice pass skipped."

#### With Graphite

```bash
# Submit PR — draft if created from main, published otherwise
if created_from_main:
    gt submit --draft --no-edit
else:
    gt submit --publish --no-edit

# Get PR URL
PR_URL=$(gh pr view --json url -q .url)

# Set crafted title and body
gh pr edit "$PR_URL" --title "[area] short description" --body "$(cat <<'EOF'
## Context
...

## Approach
...

## Reviewer guide
...

## Changes
...

## Testing
...

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

`gt submit` handles stack-aware force-pushing and PR creation. `gh pr edit` sets the structured metadata. If `gt submit` fails with a configuration or initialization error, fall back to the "Without Graphite" path below.

#### Without Graphite

```bash
if created_from_main:
    git push -u origin HEAD
    gh pr create --draft --title "[area] short description" --body "$(cat <<'EOF'
    ...template...
    EOF
    )"
else:
    gh pr create --title "[area] short description" --body "$(cat <<'EOF'
    ...template...
    EOF
    )"
```

### 5. Report

Show the user the PR URL. If the PR was created from main, note that it was submitted as a draft.

## Rules

- **Title under 70 characters** — use the description for details
- **Never auto-stage without asking** — user must see the file list and confirm before any unstaged changes are staged
- **Never force-push directly** — gt submit handles force-pushing internally; the skill must not run `git push --force`
- **Draft only for from-main PRs** — don't default to draft for PRs from existing feature branches
- **One interactive question maximum** — the unstaged changes prompt is the only question; branch creation and draft mode proceed automatically
- **Lead with the why** — reviewers care about motivation before mechanics
- **Link the design doc** — if one exists, reference it in the Approach section
- **Keep it concise** — a PR description is a summary, not documentation
