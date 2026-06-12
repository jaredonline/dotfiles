You are shipping a completed implementation as a pull request. Your goal is to produce a PR title and body from the design and implementation report, then commit, push, and open the PR directly.

## Input

Arguments come from `$ARGUMENTS`:

- **bd task id** (optional) — the tracking task to close once the ship succeeds (e.g. `bd-1234`). If present, close it with `bd close <id>` at the end.
- **ship mode** (optional) — one of:
  - `none` — commit only; no push, no PR.
  - `push` — commit and push the branch; no PR.
  - `pr` — commit, push, and open (or update) the pull request. **Default** when no mode is given.
- **design path** (optional) — absolute path to the design doc. If omitted, fall back to cockpit discovery (Step 2).

VCS strategy is auto-detected, not configured: if `gt` is on PATH use Graphite, otherwise use `gh`/`git`. There is no env-var precedence to resolve.

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Discover design doc | No — main agent | Needed for PR context |
| 2. Discover implementation report | No — main agent | Needed for PR changes summary |
| 3. Gather git context + diff | No — main agent | Branch state and the diff drive the PR |
| 4. Delegate to /pr for title+body | No — single sub-agent | Reuse /pr's voice and title conventions |
| 5. Commit, push, open PR | No — main agent | Perform the VCS action directly |
| 6. Close the task | No — main agent | `bd close <id>` if a task id was given |

## Process

### 1. Discover the design doc

If a design path was passed in `$ARGUMENTS`, use it directly.

Otherwise, fall back to cockpit grep:

```bash
BRANCH=$(git symbolic-ref --short HEAD)
BRANCH_SLUG="${BRANCH##*/}"   # strip prefix (e.g. "jared/" → "fix-auth-flow")
grep -rl "$BRANCH_SLUG" \
  "$COCKPIT_DIR/state/designs/" \
  "$COCKPIT_DIR/state/designs/finished/" \
  2>/dev/null
```

Resolution: exactly one match — use it. Multiple — pick the most recently modified. None — proceed without a design doc (the PR will still ship).

### 2. Discover the implementation report

Scan `$COCKPIT_DIR/state/implementations/` for a report referencing the bd task id (if one was passed) or the branch slug. Pick the most recently modified match. If none, proceed without an implementation report.

### 3. Gather git context and the diff

Run in parallel:

```bash
command -v gt &>/dev/null              # Graphite available?
git status
git symbolic-ref --short HEAD          # current branch name
git log main...HEAD --oneline          # commits on this branch (empty if on main)
git diff main...HEAD --stat            # files changed summary
git diff main...HEAD                   # full diff vs main
git diff --cached --name-only          # staged but uncommitted files
git diff --name-only                   # unstaged files
```

Determine whether a PR already exists for this branch (when shipping in `pr` mode):

```bash
gh pr view --json number -q .number 2>/dev/null
```

An existing PR number means we update an in-flight PR; empty means we open a new one.

### 4. Generate title and body

**Opening a new PR** (`pr` mode, no existing PR for the branch): spawn `/pr` as a sub-agent (Agent tool, `model=opus`). Pass the discovered design doc path, the implementation report path, and the diff content. Instruct the sub-agent:

- Produce only the PR title and the PR body markdown — do not run git, gh, or gt.
- Follow `/pr`'s title convention (`[area] short description`, lowercase, under 70 chars).
- Follow `/pr`'s body template (`## Context`, `## Approach`, `## Reviewer guide`, `## Changes`, `## Testing`), including the 🤖 trailer.
- Run the voice pass (`ghost-write`) before returning.
- Return the title on the first line, a blank line, then the full body markdown.

Parse the sub-agent's response into `title` and `body`. The commit message equals `title` byte-for-byte — preserve `[area]` tags, do not lowercase or strip.

**Otherwise** (`none`/`push` mode, or `pr` mode updating an existing PR): there is no new PR body to write — you only need a commit message for the new commit, and any existing PR's title and body stay put. Skip the `/pr` sub-agent and delegate to `/commit-message`: write the staged diff (`git diff --cached`, or `git diff` if nothing is staged) to a temp file, then spawn `/commit-message` as a sub-agent (Agent tool, `model=opus`) instructing it to set `COMMIT_DIFF_PATH=<tempfile>` and `COMMIT_MSG_STYLE=pr`, read the diff from that path, and return the message. Use it verbatim as the commit message.

### 5. Commit, push, and open the PR

Stage everything and commit. If there are unstaged changes, show the file list and confirm once before staging. Use the message from Step 4 verbatim.

**With Graphite:**

```bash
# Commit onto the stack (new branch from main, or amend/modify the current commit)
gt create -m "<commit_message>" -a          # if on main / new branch
# or
gt modify -a                                # if continuing an existing branch
```

**Without Graphite:**

```bash
git add -A
git checkout -b <branch-name>   # only if currently on main
git commit -m "<commit_message>"
```

Then act on the ship mode:

- **`none`** — stop here. Report the branch and commit; nothing was pushed.
- **`push`** — push the branch and stop:
  - Graphite: `gt submit --no-edit` (or `--draft --no-edit` if branched from main)
  - Git: `git push -u origin HEAD`
- **`pr`** (default) — push and open or update the PR:
  - **New PR, Graphite:**
    ```bash
    gt submit --draft --no-edit     # --publish --no-edit if not from main
    PR_URL=$(gh pr view --json url -q .url)
    gh pr edit "$PR_URL" --title "<title>" --body "<body>"
    ```
    If `gt submit` fails with a configuration/initialization error, fall back to the Git path. For auth/network errors, surface them.
  - **New PR, Git:**
    ```bash
    git push -u origin HEAD
    gh pr create --draft --title "<title>" --body "<body>"   # drop --draft if not from main
    ```
  - **Existing PR (update):** push the new commit (`gt submit --no-edit` or `git push`); leave the title and body untouched.

Draft rule: PRs opened from `main` start as drafts; PRs continuing an existing feature branch publish directly.

### 6. Close the task

If a bd task id was passed in `$ARGUMENTS`, close it now:

```bash
bd close <id>
```

Then report the result: branch, commit subject, and the PR URL (if one was opened or updated).

## Rules

- **Title under 70 characters** — preserve `[area]` tags; the commit message equals the title verbatim.
- **Never auto-stage without asking** — show the unstaged file list and confirm once before staging.
- **Never force-push directly** — `gt submit` handles force-pushing; do not run `git push --force`.
- **Draft only for from-main PRs** — branches off an existing feature branch publish directly.
- **Delegate to `/pr` for voice and formatting on the new-PR path** — do not reimplement title or body rules here. The existing-PR update path skips `/pr` and only generates a commit message via `/commit-message`.
- **Respect the ship mode** — `none` commits only, `push` pushes, `pr` opens/updates the PR. Do nothing beyond the requested mode.
- **Close the task only after the ship succeeds** — `bd close <id>` runs last, and only if an id was given.
