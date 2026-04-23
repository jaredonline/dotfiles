---
name: feedback-explore
description: Apply feedback to an exploration document preserving structural invariants
user_invocable: false
---

**When running under krust** ($KRUST_BEADS_ID is set):

Read the artifact path and feedback from beads:
- Run: `bd show $KRUST_BEADS_ID --json`
- Extract `artifact_path` from `.metadata.krust.artifact_path`
- Extract latest feedback from `.notes` (last line)

**When running standalone** ($KRUST_BEADS_ID is not set, fallback):

- Read the file at $KRUST_OUT
- If $KRUST_FEEDBACK is set and non-empty, use it as the feedback

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

Do NOT skip these checks — they're the structural contract for exploration docs.

After rewriting the file, signal completion:
```bash
if [ -n "$KRUST_BEADS_ID" ]; then
  bd update $KRUST_BEADS_ID --set-metadata='artifact_written=true'
  bd update $KRUST_BEADS_ID --set-metadata='skill_complete=true'
fi
```
