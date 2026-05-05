---
name: feedback-explore
description: Apply feedback to an exploration document preserving structural invariants
user_invocable: false
---

Read the artifact at `$KRUST_OUT`.

The feedback text is `$KRUST_FEEDBACK` (always set; may be empty). If empty, the user picked `[E]dit` and edited the file by hand — no feedback text to apply, review their edits for consistency only.

---

Read the artifact file.

If feedback text is available, apply it. Otherwise, the user just edited the file by hand — review their edits and ensure internal consistency.

Rewrite the file in place.

After applying changes, verify these invariants and fix any violations:

1. **Section completeness**: Keep every section from the `/explore` template — Overview, Architecture, Request/Data Flows, Data Stores & Schemas, Integration Points, Key Invariants, Failure Modes, Tracking. Do not drop sections even if feedback doesn't touch them.
2. **Evidence discipline**: every factual claim must cite a specific file and function/line. Inferences must be labeled ("the column name suggests…"), not restated as facts.
3. **Architecture diagram**: the Mermaid diagram matches the components described in the text. If feedback added or removed a component, update the diagram too.
4. **Data flows are end-to-end**: each numbered flow has an entry point (file:line), steps with file references, and a terminal state. Fragment-only flows are a regression.
5. **Integration points list direction + protocol**: every inbound/outbound entry names the direction and the protocol.
6. **Failure modes are concrete**: each entry names what breaks and how the system handles it, not vague risk statements.
7. **Tracking preserved**: `## Tracking` section still lists the Beads task ID. Do not rewrite the task ID.
8. **Preserve `## Rounds of Feedback` section**: If a top-level section titled exactly `## Rounds of Feedback` exists at the end of the artifact, preserve it byte-for-byte. Do not edit, reformat, reorder, summarize, move, or remove any content within or after it. Krust will append the new round entry after you return; do not add a round entry yourself.

Do NOT skip these checks — they're the structural contract for exploration docs.

After rewriting the file, signal completion:
```bash
if [ -n "$KRUST_BEADS_ID" ]; then
  bd update $KRUST_BEADS_ID --set-metadata='artifact_written=true'
  bd update $KRUST_BEADS_ID --set-metadata='skill_complete=true'
fi
```
