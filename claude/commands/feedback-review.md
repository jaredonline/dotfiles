---
name: feedback-review
description: Apply feedback to a review artifact preserving structural invariants
user_invocable: false
---

Read the artifact at `$KRUST_OUT`.

The feedback text is `$KRUST_FEEDBACK` (always set; may be empty). If empty, the user picked `[E]dit` and edited the file by hand — no feedback text to apply, review their edits for consistency only.

---

Read the artifact file.

If feedback text is available, apply it. Otherwise, the user just edited the file by hand — review their edits and ensure internal consistency.

Rewrite the file in place at `$KRUST_OUT`.

After applying changes, verify these invariants and fix any violations:

1. **Section order and completeness**: Sections are present in this fixed order — `Summary`, `Priority (Consensus ≥ 2)`, `Critical`, `High`, `Medium`, `Low`, `False Positives Removed`. Do not drop sections even if a severity bucket is empty.
2. **Finding heading format**: every finding heading is `### [<ID>] <title>` where ID is `FC<n>` (Critical), `FH<n>` (High), `FM<n>` (Medium), `FL<n>` (Low), or `FP<n>` (Priority).
3. **Finding body completeness**: every finding has a severity label, a file:line reference in an inline code span, a description, and at least one persona attribution (`from: ...` line).
4. **Priority section scope**: `Priority (Consensus ≥ 2)` contains only findings flagged by ≥ 2 personas. Priority entries use `FP<n>` IDs and reference an underlying severity finding — they never introduce new content that isn't present in Critical/High/Medium/Low.
5. **False Positives Removed**: entries excluded by the correctness filter stay here with persona + reason intact.
6. **ID stability**: IDs assigned on initial generation never change. When feedback removes a finding, its ID is NOT reused. New findings added by feedback get the next free number in their severity bucket (scan existing IDs, pick max + 1). Renumbering is PROHIBITED.
7. **Approval state consistency**: frontmatter `approved_findings` is preserved verbatim unless feedback explicitly removes a finding whose ID is in the list — in which case drop from BOTH the list AND the body together. Body `> ✅ **Approved for resolution**` markers stay on findings that remain. The document must be self-consistent on exit.
8. **Frontmatter preservation**: `name`, `beads_id`, `mode`, `dirty`, `head_sha`, `base_ref`, `pr_url`, `reviewed_at` are NEVER modified by feedback. `head_sha` in particular is the staleness signal — feedback does not refresh it. `approved_findings` is only modified consistently with body edits per invariant 7.

Do NOT skip these checks — they're the structural contract for review artifacts.

After rewriting the file, signal completion:
```bash
if [ -n "$KRUST_BEADS_ID" ]; then
  bd update $KRUST_BEADS_ID --set-metadata='artifact_written=true'
  bd update $KRUST_BEADS_ID --set-metadata='skill_complete=true'
fi
```
