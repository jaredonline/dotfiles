---
name: feedback-explore
description: Apply feedback to an exploration document preserving structural invariants
user_invocable: false
---

Apply user feedback to an existing exploration document, preserving its structural invariants.

`$ARGUMENTS` carries the artifact path and the feedback text. The artifact path is the exploration doc to edit (typically under `$COCKPIT_DIR/state/...`); the feedback text is the change to apply. If no feedback text is given, the user edited the file by hand — review their edits for internal consistency only, applying no new changes.

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
8. **Preserve `## Brief` byte-for-byte**: If a top-level section titled exactly `## Brief` exists, preserve it byte-for-byte — the original brief stays the original brief; feedback is feedback, not a new brief. If `## Brief` is absent (empty-brief artifact), do not add a placeholder. Do not edit, reformat, reorder, summarize, move, or remove any content within it.
9. **Manage `## Rounds of Feedback` yourself**: A top-level `## Rounds of Feedback` section lives at the end of the artifact and logs each feedback round. Preserve every existing round entry byte-for-byte — do not edit, reformat, reorder, or remove prior entries. After applying this round's feedback, append a new round entry to the end of that section recording the feedback you just applied. If the section does not exist yet (and feedback was applied), create it as the final top-level section before adding the first entry. Use this format for each entry:
   ```markdown
   ### Round N — YYYY-MM-DD
   <the feedback text applied this round, and a one-line note on what changed>
   ```
   Number rounds sequentially (the next integer after the last existing entry, or 1 if none). When the user only hand-edited the file (no feedback text), do not add a round entry.

Do NOT skip these checks — they're the structural contract for exploration docs.
