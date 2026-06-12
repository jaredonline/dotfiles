---
name: approve-review
description: Amend a review artifact with approval markers for a list of finding IDs
user_invocable: false
---

Amend a review artifact in place by marking a set of findings as approved for resolution. The skill validates the incoming ID list before any mutation, inserts a `> ✅ **Approved for resolution**` marker directly under each finding heading, updates the frontmatter `approved_findings` list to the union of prior and incoming IDs (sorted severity-descending, then numeric), and commits the amended artifact.

## Inputs

- The review artifact path — `$COCKPIT_DIR/state/reviews/review-<slug>.md` (the artifact produced by `/review`). The `<slug>` and comma-separated finding IDs come from `$ARGUMENTS` (e.g. `auth-refactor FC1,FH2,FH3`). If the slug is ambiguous, list the artifacts under `$COCKPIT_DIR/state/reviews/` and ask.
- The artifact frontmatter carries `beads_id` (the review task created by `/review`).

## Steps

1. Resolve the artifact path and parse the approved-ID list from `$ARGUMENTS`. Trim whitespace around each token and tolerate empty tokens (skip them).
2. Read the artifact.
3. **Pre-validate** the incoming list BEFORE any mutation:
   - If any ID starts with `FP`, abort with: `approve-review: cannot approve Priority reference ID <ID> — approve the underlying severity ID instead`.
   - For each ID, locate its `### [<ID>] <title>` heading in the body. If any ID is absent, abort with: `approve-review: ID <ID> not found in artifact`. Do NOT write a partial amendment.
4. For each validated ID:
   - If the next non-empty line after the heading is NOT already `> ✅ **Approved for resolution**`, insert that line directly under the heading followed by a blank line.
   - If the marker is already present, skip (idempotent).
5. Update frontmatter `approved_findings`:
   - Parse the existing list.
   - Set it to the UNION of the existing list and the incoming IDs.
   - Order by severity descending (C > H > M > L), then numeric ascending within severity. Example: `[FC1, FC2, FH1, FH3, FM1]`.
   - Preserve all other frontmatter fields verbatim.
6. Write the amended markdown back to the artifact path.
7. Commit and push the amendment:
   ```bash
   git -C "$COCKPIT_DIR" add "state/reviews/review-<slug>.md"
   git -C "$COCKPIT_DIR" commit -m "approve-review: <slug> — approved <ids>"
   git -C "$COCKPIT_DIR" push
   ```

## Invariants

- **Idempotent**: running twice with the same inputs yields byte-identical output after the first call.
- **Reject `FP<n>` IDs**: if any incoming ID starts with `FP`, abort with `approve-review: cannot approve Priority reference ID <ID> — approve the underlying severity ID instead`. Priority entries are views, not findings.
- **Self-consistency**: every ID listed in `approved_findings` has a marker in the body, and every marker in the body has its ID listed in `approved_findings`.
- **No finding IDs changed**: the skill only adds a marker line and updates the frontmatter list; it never rewrites or reorders finding blocks.
- **No body edits beyond marker insertion**: section headings, descriptions, and all other content are untouched.
- **Frontmatter fields preserved**: `name`, `beads_id`, `head_sha`, `base_ref`, `pr_url`, `reviewed_at` are all untouched.
