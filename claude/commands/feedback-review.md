---
name: feedback-review
description: Apply feedback to a review artifact preserving structural invariants
user_invocable: false
---

Apply feedback to a review artifact in place, preserving its structural invariants, then commit.

## Inputs

- The review artifact path — `$COCKPIT_DIR/state/reviews/review-<slug>.md` (the artifact `/review` produced). The `<slug>` and the feedback text come from `$ARGUMENTS` (e.g. `auth-refactor downgrade FH2 to Medium, it's guarded by the nil check`). If the slug is ambiguous, list the artifacts under `$COCKPIT_DIR/state/reviews/` and ask.
- If `$ARGUMENTS` carries no feedback text (just a slug), the user edited the file by hand — there is no feedback text to apply, so review their edits for consistency only.

## Steps

Read the artifact file.

If feedback text is available, apply it. Otherwise, the user just edited the file by hand — review their edits and ensure internal consistency.

Rewrite the file in place at the review artifact path.

After applying changes, verify these invariants and fix any violations:

1. **Section order and completeness**: Sections are present in this fixed order — `Summary`, `Priority (Consensus ≥ 2)`, `Critical`, `High`, `Medium`, `Low`, `False Positives Removed`. Do not drop sections even if a severity bucket is empty.
2. **Finding heading format**: every finding heading is `### [<ID>] <title>` where ID is `FC<n>` (Critical), `FH<n>` (High), `FM<n>` (Medium), `FL<n>` (Low), or `FP<n>` (Priority).
3. **Finding body completeness**: every finding has a severity label, a file:line reference in an inline code span, a description, and at least one persona attribution (`from: ...` line).
4. **Priority section scope**: `Priority (Consensus ≥ 2)` contains only findings flagged by ≥ 2 personas. Priority entries use `FP<n>` IDs and reference an underlying severity finding — they never introduce new content that isn't present in Critical/High/Medium/Low.
5. **False Positives Removed**: entries excluded by the correctness filter stay here with persona + reason intact.
6. **ID stability**: IDs assigned on initial generation never change. When feedback removes a finding, its ID is NOT reused. New findings added by feedback get the next free number in their severity bucket (scan existing IDs, pick max + 1). Renumbering is PROHIBITED.
7. **Approval state consistency**: frontmatter `approved_findings` is preserved verbatim unless feedback explicitly removes a finding whose ID is in the list — in which case drop from BOTH the list AND the body together. Body `> ✅ **Approved for resolution**` markers stay on findings that remain. Terminal `> ✅ **Fixed (round …)**` / `> ❌ **Fix failed (round …)**` markers and any `## Fix Outcomes (round N)` sections are preserved verbatim. The document must be self-consistent on exit.
8. **Frontmatter preservation**: `name`, `beads_id`, `head_sha`, `base_ref`, `pr_url`, `reviewed_at` are NEVER modified by feedback. `head_sha` in particular is the staleness signal — feedback does not refresh it. `approved_findings` is only modified consistently with body edits per invariant 7.
9. **Preserve `## Brief` section**: If a top-level section titled exactly `## Brief` exists, preserve it byte-for-byte — the original brief stays the original brief; feedback is feedback, not a new brief. Do not edit, reformat, reorder, summarize, move, or remove any content within it. If `## Brief` is absent, do not add a placeholder.

Do NOT skip these checks — they're the structural contract for review artifacts.

After rewriting the file, commit and push the amendment:
```bash
git -C "$COCKPIT_DIR" add "state/reviews/review-<slug>.md"
git -C "$COCKPIT_DIR" commit -m "feedback-review: <slug>"
git -C "$COCKPIT_DIR" push
```

The amended artifact routes back into the loop: re-run `/fix-review` to apply patches for any (re-)approved findings, or hand back to the user for further `/approve-review` / `/feedback-review` passes.
