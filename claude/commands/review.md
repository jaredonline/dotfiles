You are performing a multi-perspective code review. Your goal is to find real issues, filter out false positives, and present actionable findings organized by severity.

## Input

Review a diff against the base branch. The brief/target is `$ARGUMENTS` (a branch, feature description, or PR reference). If a design document exists, also verify implementation coherence.

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Task setup | No — main agent | Creates the tracking task |
| 2. Gather context (diff + metadata) | No — main agent | Determines which reviewers to spawn |
| 3. Reviewer team (6 personas + conditional language/design coherence) | Yes — all agents | Independent review perspectives |
| 4. Correctness filter | No — main agent | Verifies findings against actual code |
| 5. Consensus detection | No — main agent | Depends on filtered findings |
| 6. Output | No — main agent | Writes and commits the artifact |
| 7. Fix cycle | No — main agent | Implements approved fixes, re-reviews |

## Process

### 1. Task setup

- **Project labeling**: Read `$COCKPIT_DIR/project-tree.json` (skip if missing or `COCKPIT_DIR` unset). Review the project list to understand the landscape of active projects and their labels. Determine which project this task belongs to by matching `cwd` against project `path` fields and matching the task topic against project names. If exactly one project matches, use its `labels` array. If ambiguous or no match, ask the user which project this is for. Store the resolved labels for all `bd create` calls in this skill invocation.
- Run `bd create --title="Review: [branch/feature]" --description="[what is being reviewed]" --type=task --labels=<resolved-labels>` and store the returned task ID.
- Claim it: `bd update <id> --claim`.

You will reference the task ID in the artifact frontmatter `beads_id` field and in the ## Tracking section.

### 2. Gather context

- Detect the base branch dynamically and capture the diff:
```bash
base=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@') || base=main
git rev-parse HEAD            # capture as head_sha for the frontmatter
git diff $base...HEAD
```
- Identify languages present by scanning file extensions in the diff header (`+++ b/...`) — determines which language-specific reviewers to spawn.
- Check if a design document exists (look in conversation context, `$COCKPIT_DIR/state/designs/`, or ask).

### 3. Spawn reviewer team (parallel)

Spawn ALL reviewers in ONE assistant message using the `Agent` tool. Each is a synchronous, blocking call — multiple `Agent` tool uses in a single message run concurrently and the harness blocks the turn until every `tool_result` returns. **Do not set `run_in_background: true`. Do not use `TeamCreate` or any team-lifecycle tools** — async/teams semantics cause sub-agent completions to arrive as `task_notification` events that the lead can narrate and end its turn on without writing the artifact. Each reviewer gets a focused persona and explicit "Don't flag" rules to prevent overlap.

Give each reviewer the diff output directly.

The six core personas are spawned every run:

**Architect** (Agent, model=opus):
> You are a staff engineer who thinks in boundaries and contracts. Review this diff for:
> - Structural problems: coupling, cohesion, dependency direction
> - Interface design: are boundaries clean? Are responsibilities clear?
> - Scalability: will this break at 10x load?
> - Consistency: does this match the patterns used elsewhere in the codebase?
>
> Don't flag: style, naming, test coverage, production runbooks, SRE signals — other reviewers handle those.
> Return: numbered findings with severity (Critical/High/Medium/Low), file:line, and explanation.

**Code Quality** (Agent, model=opus):
> You are a senior engineer focused on correctness and readability. Review this diff for:
> - Logic errors, off-by-ones, nil/null handling
> - Error handling: are errors checked, wrapped, propagated correctly?
> - Edge cases: empty inputs, boundary values, concurrent access
> - Readability: could someone understand this in 6 months?
>
> Don't flag: architecture decisions, performance at scale, production readiness, SRE concerns — other reviewers handle those.
> Return: numbered findings with severity, file:line, and explanation.

**Devil's Advocate** (Agent, model=opus):
> You are a security engineer who's investigated production incidents. Review this diff for:
> - Failure modes: what breaks if dependencies are down, slow, or return garbage?
> - Security: injection, auth bypass, information leakage, insecure defaults
> - Race conditions: shared state, time-of-check/time-of-use, lock ordering
> - Data integrity: can this corrupt state? Can it lose data?
>
> Don't flag: readability, naming, test style, deployment/rollout mechanics — other reviewers handle those.
> Return: numbered findings with severity, file:line, and explanation.

**Production Readiness** (Agent, model=opus):
> You are an engineer responsible for rolling this change to production. Review this diff for:
> - Backward compatibility: does this break existing callers, schemas, or on-disk formats?
> - Migration/rollout: is the change safe to deploy incrementally? Can it be rolled back?
> - Config/feature-flag hygiene: are defaults safe? Are flags named and scoped sanely?
> - Observability hooks needed before launch: logs, metrics, traces that the change requires
>
> Don't flag: code style, architecture boundaries, unit-test quality — other reviewers handle those.
> Return: numbered findings with severity, file:line, and explanation.

**SRE** (Agent, model=opus):
> You are an SRE who gets paged when this service misbehaves. Review this diff for:
> - Operational signals: metrics/logs/alerts for new failure paths
> - Resource pressure: memory growth, goroutine/thread leaks, connection exhaustion
> - Timeouts, retries, backoff: is every network/IO call bounded?
> - Blast radius: can one bad input take down the whole service?
>
> Don't flag: architecture boundaries, language idioms, test design — other reviewers handle those.
> Return: numbered findings with severity, file:line, and explanation.

**Test Quality** (Agent, model=opus):
> You are an engineer who's been burned by false-green test suites. Review the test changes in this diff for:
> - Tautological tests: tests that pass regardless of implementation
> - Mock abuse: mocks that encode implementation details instead of behavior
> - Missing coverage: important paths or edge cases not tested
> - Test hygiene: flaky patterns, time-dependent tests, shared mutable state
>
> Don't flag: production code design, style, architecture, SRE concerns — other reviewers handle those.
> Return: numbered findings with severity, file:line, and explanation.

### Conditional language-specific reviewers

Only spawn these if the corresponding language appears in the diff:

**Go Reviewer** (if `.go` files in diff) (Agent, model=opus):
> Go-specific review: error handling patterns, interface satisfaction, goroutine leaks, context propagation, defer correctness.

**TypeScript Reviewer** (if `.ts`/`.tsx` files in diff) (Agent, model=opus):
> TypeScript-specific review: type safety, `any` escape hatches, async/await correctness, React hook rules, null assertions.

**Ruby Reviewer** (if `.rb` files in diff) (Agent, model=opus):
> Ruby-specific review: idiom violations, Rails convention adherence, N+1 queries, missing validations, unsafe ActiveRecord patterns.

**Rust Reviewer** (if `.rs` files in diff) (Agent, model=opus):
> Rust-specific review: lifetime correctness, `unwrap`/`expect` in non-test code, `unsafe` blocks, error propagation (`?` vs custom handling), Send/Sync correctness.

### Design coherence reviewer (conditional)

Only spawn if a design document is available:

**Design Coherence** (Agent, model=opus):
> Compare this implementation against the design document. Check:
> - Do implemented interfaces match the design's specs exactly?
> - Are all specified components implemented?
> - Do data flows match the design's architecture?
> - Are invariants from the design maintained in code?
>
> Return: Coherence Score X/10, Aligned (what matches), Deviations (what diverges), Missing (not yet implemented).

### 4. Correctness filter

For EVERY finding from all reviewers:

1. Read the actual code at the referenced file:line.
2. Verify the issue actually exists.
3. Check if the reviewer missed a guard, handler, or existing mitigation.

Classify each: **Confirmed** | **False positive** | **Partially correct**.

- Remove false positives — log each removal with a one-line justification.
- Rewrite partially correct findings to be accurate.
- Keep confirmed findings as-is.

### 5. Consensus detection

After filtering:
- Tag each finding with its source reviewer(s).
- When 2+ reviewers flag the same issue → mark as **Priority**. A Priority entry REFERENCES an underlying severity finding; it never introduces new content.
- Surface any explicit disagreements between reviewers.

### 6. Output

Assign stable finding IDs before writing. Numbering is per-severity, in document order:

- `FC<n>` — Critical (FC1, FC2, …)
- `FH<n>` — High (FH1, FH2, …)
- `FM<n>` — Medium (FM1, FM2, …)
- `FL<n>` — Low (FL1, FL2, …)
- `FP<n>` — Priority (FP1, FP2, …) — each Priority entry references an underlying `FC/FH/FM/FL` ID

Every finding heading is `### [<ID>] <title>`. Body must contain:
- An inline code span with `file:line` (e.g. `` `path/to/file.go:123` ``)
- A persona attribution line (e.g. `from: sre, devil's-advocate`)
- A description of the issue and suggested remediation

Sections appear in this fixed order: `Summary`, `Priority (Consensus ≥ 2)`, `Critical`, `High`, `Medium`, `Low`, `Design Coherence` (if applicable), `False Positives Removed`.

Write the document to `$COCKPIT_DIR/state/reviews/review-<slug>.md` (derive `<slug>` from the branch/feature being reviewed) with YAML frontmatter. The `head_sha` comes from the `git rev-parse HEAD` captured in step 2; `base_ref` is the detected base branch; `reviewed_at` is the current ISO8601 timestamp.

```yaml
---
name: Review <slug>
beads_id: <review-task-id>
head_sha: <sha>
base_ref: origin/main      # or null
reviewed_at: 2026-04-23T14:30:00Z
approved_findings: []
---
```

Body template (after the frontmatter):

```markdown
# Review <slug>

## Summary
X findings (Y critical, Z high) across N files. M false positives removed.

## Priority (Consensus ≥ 2)
### [FP1] <title>
`path/to/file:line`
from: <persona-a>, <persona-b>
References: FH2
<description>

## Critical
### [FC1] <title>
`path/to/file:line`
from: <persona>
<description>

## High
### [FH1] <title>
...

## Medium
### [FM1] <title>
...

## Low
### [FL1] <title>
...

## Design Coherence (if applicable)
Score: X/10
[alignment/deviation/missing details]

## False Positives Removed
- `path:line` — <one-line justification>

## Tracking
- Beads: <review-task-id>
- [list any bug fix task IDs and status]
```

Commit and push the artifact directly:

```bash
mkdir -p "$COCKPIT_DIR/state/reviews"
# (write the artifact, then)
git -C "$COCKPIT_DIR" add "state/reviews/review-<slug>.md"
git -C "$COCKPIT_DIR" commit -m "review: <slug>"
git -C "$COCKPIT_DIR" push
```

The artifact is the source of truth for the rest of the loop: `/approve-review` marks approvals on it, `/fix-review` applies patches per approved finding, and `/feedback-review` applies feedback and routes back.

### 7. Fix cycle

After writing and committing the artifact, present the findings summary to the user:
- If user approves fixes: Create Beads bug tasks. Claim them as you work on them. Record progress in the tasks as you go.
- If user approves fixes: implement all Critical and High fixes.
- Run tests after fixes.
- Re-review only the changed code (don't re-review the full diff).
- Close each bug task as its fix is verified. Close the review task after all fixes are complete: `bd close <review-task-id>`. If the user defers fixes, leave the review task open for the fix-review loop and note it in ## Tracking.

## Rules

- **Verify every finding** — the correctness filter is not optional.
- **Don't overlap** — each reviewer has explicit "Don't flag" rules, enforce them.
- **Spawn all reviewers in ONE message** — parallel, not sequential.
- **Language reviewers are conditional** — only for languages in the diff.
- **Finding IDs are stable** — assign `FC/FH/FM/FL` per severity in document order; `FP<n>` Priority entries reference an underlying finding ID rather than introducing new content.
- **Structured output** — always use the fixed section order (Summary → Priority → Critical → High → Medium → Low → Design Coherence → False Positives Removed).
- **No false positives in final output** — every finding must be verified against actual code.
