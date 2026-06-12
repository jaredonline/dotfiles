---
name: fix-review
description: Apply minimal code patches per approved review finding and emit a per-round fix report
model: opus
user_invocable: false
---

Apply minimal code patches for the approved findings on a review artifact. Each approved finding becomes one worker subagent that produces the smallest possible patch addressing the finding, runs the project test command, and reports back. The lead aggregates outcomes, mutates the review artifact's approval markers in place, writes a summary-only fix report, and commits.

## Inputs

- The review artifact path — `$COCKPIT_DIR/state/reviews/review-<slug>.md` (the artifact `/review` produced and `/approve-review` amended). The `<slug>` comes from `$ARGUMENTS`. If ambiguous, list the artifacts under `$COCKPIT_DIR/state/reviews/` and ask.
- The set of approved finding IDs is read from the artifact frontmatter `approved_findings` list (the IDs `/approve-review` marked). `$ARGUMENTS` may also pass an explicit comma-separated subset (e.g. `auth-refactor FC1,FH2`); when omitted, fix every approved finding that is not already terminal.
- The artifact frontmatter carries `beads_id` (the review task) and `head_sha` (the staleness signal).

## Steps

### 1. Validate inputs and preconditions

Read the review artifact. Parse the YAML frontmatter and capture `beads_id`, `head_sha`, the `approved_findings` list, and (if present) `round`.

Resolve the list of IDs to fix: the explicit subset from `$ARGUMENTS` if given, otherwise the full `approved_findings` list. Trim whitespace, drop empty tokens.

If the resulting list is empty, abort:
```bash
echo "fix-review: no approved findings to fix" >&2
exit 1
```

Compare the artifact's `head_sha` against the current HEAD:
```bash
current=$(git rev-parse HEAD)
```

If `current != head_sha` from the artifact frontmatter, abort:
```
fix-review: review staleness — current HEAD != review.head_sha, re-run /review against current HEAD
```

### 2. Derive round number `<n>`

Resolve `<n>` in this order:
1. If the review artifact frontmatter contains a `round` field, use that integer.
2. Otherwise, scan the artifact body for existing `## Fix Outcomes (round N)` sections. Use `max(N) + 1`. If none exist, use `1`.

`<n>` is stamped into the artifact's appended Fix Outcomes section, into each rewritten marker, and into the fix report frontmatter.

### 3. Locate findings and skip terminal markers

For each approved ID, locate its `### [<ID>] <title>` heading in the artifact body. Capture the full finding body (heading through the next `### ` heading or next `## ` section boundary).

Inspect the marker block directly under the heading. Skip the ID entirely (do not spawn a worker, do not mutate the marker) if the existing marker is already terminal:
- `> ✅ **Fixed (round …)** …`
- `> ❌ **Fix failed (round …)** …`

This idempotency rule ensures crash-recovery re-runs don't re-attempt or stomp terminal outcomes from prior rounds.

For each non-terminal approved finding, extract:
- **Primary file**: the path from the inline `` `path/to/file:line` `` reference in the body.
- **Additional files**: any other file paths explicitly named in the finding body text. The body itself is the worker brief — there is no separate "suggested fix" subsection to parse.

### 4. Group and spawn workers

Group findings by primary file. Scheduling rules:
- **Within a group** (same primary file): run findings serially. Two workers editing the same file in parallel would race on edits.
- **Across groups** (different primary files): run in parallel. Spawn all cross-group workers in ONE assistant message with multiple `Agent` tool calls.

Workers are spawned via the `Agent` tool with `model=opus` and `isolation=none` (workers share the working tree). Within a single file's group, the lead waits for the prior worker to finish before launching the next.

Each worker receives the EXACT prompt below, with `<ID>`, the pasted finding heading + body, and the scope file list substituted:

```
You are fixing one approved review finding.

## Finding [<ID>]
<paste finding heading + body from review artifact>

## Scope (HARD LIMIT)
You may only modify these files:
- <primary file from finding path:line>
- <any other file explicitly named in the finding body>

If addressing the finding requires editing any other file, do not edit it. Revert anything you touched and return STATUS: fix_failed with REASON: "out of scope — would need to edit <file>".

## Rules
- Minimal patch — only what's needed to address the finding as stated.
- No refactors, no abstractions, no "while I'm here" cleanups.
- Write or update a test that demonstrates the fix when the finding is testable.
- Run the project's test command (detect: cargo test / go test ./... / npm test / pytest) after editing. It must pass.
- Do not run git commands. The lead handles commits.
- Do not retry. If your first edit attempt doesn't get tests green, revert your edits (delete new content; restore originals from your initial Read) and return STATUS: fix_failed with the failing test output as the reason.

## Output (last line of your response MUST be one of)
STATUS: fixed
SUMMARY: <one line for the artifact marker>
FILES: <comma-separated list of files you modified>

STATUS: fix_failed
REASON: <one line>
```

### 5. Classify worker outcomes

For each worker, parse the final `STATUS:` line:

- `STATUS: fixed` → capture `SUMMARY:` and `FILES:` lines. Validate the `FILES:` list against the declared scope (primary file + body-named files). If any reported file is outside scope, downgrade to a failure with reason `"scope violation: modified <file> outside declared scope"` even if the worker's tests passed.
- `STATUS: fix_failed` → capture the `REASON:` line.
- No parseable `STATUS:` line → treat as failure with reason `"worker produced no STATUS line"`.

### 6. Final smoke test

If any worker reported `STATUS: fixed` and survived scope validation, run the project's test command once more from the lead (final smoke run, detect: cargo test / go test ./... / npm test / pytest).

If the final smoke fails, mark the overall fix report run as `partial` but DO NOT revert the successful per-finding edits — the successful per-finding fixes are retained regardless. Surface the smoke failure in the fix report.

### 7. Mutate the review artifact in place

For every finding the skill processed this round, replace its `> ✅ **Approved for resolution**` marker (the block sits directly under the heading) with the appropriate terminal marker:

- On success:
  ```
  > ✅ **Fixed (round <n>)** — <one-line summary>
  ```
- On failure:
  ```
  > ❌ **Fix failed (round <n>)** — <reason>
  ```

Marker mutation rules:
- **Replacement, not stacking**: the original `> ✅ **Approved for resolution**` line is REPLACED by the terminal marker. Do not leave both in place.
- **Monotonicity**: each finding transitions Approved → Fixed or Approved → FixFailed exactly once per round. Once a marker is terminal, the skill never touches it again on re-run (step 3's skip rule enforces this).
- **No body edits beyond the marker**: finding bodies, IDs, headings, and section ordering are otherwise untouched.

Append a new section to the artifact (after any existing Fix Outcomes sections, never rewriting prior rounds):

```markdown
## Fix Outcomes (round <n>)
- [FC1] ✅ Fixed — <summary> — files: path/a.rs
- [FH2] ✅ Fixed — <summary> — files: path/a.rs
- [FH3] ❌ Fix failed — <reason>
```

Write the modified artifact back to the review artifact path.

### 8. Write the fix report

Write a per-round fix report to `$COCKPIT_DIR/state/reviews/fix-review-<slug>-round<n>.md`. The fix report is summary-only — finding bodies and per-finding descriptions stay in the review artifact, which is the source of truth.

```markdown
---
name: Fix Review <slug>
beads_id: <id>
review_artifact: review-<slug>.md
round: <n>
head_sha: <sha matching review.head_sha at the time of fix>
applied_at: <iso8601>
---

# Fix Review <slug> (round <n>)

N approved · X fixed · Y failed

## Outcomes
- FC1: fixed
- FH2: fixed
- FH3: failed — see review artifact for finding body and `❌ Fix failed` marker reason
```

If the final smoke run failed, add a single line under the count summary: `Smoke test: failed — run marked partial (successful per-finding edits retained).`

### 9. Commit

Commit the code patches and the updated review artifact + fix report directly:
```bash
git add -A
git commit -m "fix-review: <slug> round <n> — X fixed, Y failed"
git -C "$COCKPIT_DIR" add "state/reviews/review-<slug>.md" "state/reviews/fix-review-<slug>-round<n>.md"
git -C "$COCKPIT_DIR" commit -m "fix-review: <slug> round <n> report"
```

Push both. If `$COCKPIT_DIR` is the same repo as the working tree, collapse into a single add/commit/push.

If failed findings remain, route them back through `/feedback-review` (to revise the finding or its scope) and re-run; otherwise close the review task once every approved finding is terminal-fixed: `bd close <beads_id>`.

## Invariants

- Resolved fix list empty ⇒ abort before any read/mutation.
- HEAD-SHA mismatch ⇒ abort before any worker spawn or artifact mutation.
- Terminal markers from prior rounds are never rewritten.
- Approved marker is replaced (not stacked) exactly once per round per finding.
- Workers run serially within a file group, in parallel across file groups (one assistant message with multiple Agent calls).
- Workers operate with `model=opus` and `isolation=none` (shared working tree).
- Worker `FILES:` outside declared scope ⇒ treated as failure regardless of test outcome.
- Fix report is summary-only; finding bodies stay in the review artifact.
- Successful per-finding edits are retained even when the final smoke run fails (run marked partial).
