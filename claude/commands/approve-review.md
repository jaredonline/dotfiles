---
name: approve-review
description: Amend a review artifact with approval markers for a list of finding IDs
user_invocable: false
---

Amend the review artifact at `$KRUST_OUT` in place by marking a set of findings as approved for resolution. The skill validates the incoming ID list before any mutation, inserts a `> ✅ **Approved for resolution**` marker directly under each finding heading, and updates the frontmatter `approved_findings` list to the union of prior and incoming IDs (sorted severity-descending, then numeric).

## Inputs

- `$KRUST_BEADS_ID` — bd task id
- `$KRUST_OUT` — path to the existing review artifact to amend in place
- `$KRUST_APPROVED_IDS` — comma-separated list of finding IDs, e.g. `"FC1,FH2,FH3"`
- `$ACTIONS_DIR` — directory for action files

## Steps

1. Parse `$KRUST_APPROVED_IDS` into a list of IDs. Trim whitespace around each token and tolerate empty tokens (skip them).
2. Read the artifact at `$KRUST_OUT`.
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
6. Write the amended markdown back to `$KRUST_OUT`.
7. Signal completion: `bd update $KRUST_BEADS_ID --set-metadata='skill_complete=true'`.

## Invariants

- **Idempotent**: running twice with the same inputs yields byte-identical output after the first call.
- **Reject `FP<n>` IDs**: if any incoming ID starts with `FP`, abort with `approve-review: cannot approve Priority reference ID <ID> — approve the underlying severity ID instead`. Priority entries are views, not findings.
- **Self-consistency**: every ID listed in `approved_findings` has a marker in the body, and every marker in the body has its ID listed in `approved_findings`.
- **No finding IDs changed**: the skill only adds a marker line and updates the frontmatter list; it never rewrites or reorders finding blocks.
- **No body edits beyond marker insertion**: section headings, descriptions, and all other content are untouched.
- **Frontmatter fields preserved**: `name`, `beads_id`, `mode`, `dirty`, `head_sha`, `base_ref`, `pr_url`, `reviewed_at` are all untouched.
