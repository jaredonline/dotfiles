---
name: feedback-situation
description: Apply feedback to a situation design document preserving structural invariants
user_invocable: false
---

Read the artifact at `$KRUST_OUT`.

The feedback text is `$KRUST_FEEDBACK` (always set; may be empty). If empty, the user picked `[E]dit` and edited the file by hand — no feedback text to apply, review their edits for consistency only.

---

Read the artifact file.

If feedback text is available, apply this feedback.
Otherwise, the user just edited this file by hand. Review the edits and ensure internal consistency.

Rewrite the file in place.

After applying the changes, verify the following invariants and fix any violations:

1. **Faction completeness**: Every faction has all required fields (identity,
   area of operation, power level, ideology, methods, goals at 3 timescales,
   key NPCs table, clock).
2. **NPC goal distinction**: Every NPC's personal goal is distinct from their
   faction's goals.
3. **Faction relationship map (Mermaid)**: Matches the actual factions and their
   relationships in the document.
4. **PC goal intersections table**: Covers all factions (if PC goals were
   provided).
5. **Default timeline**: Includes at least one entry per faction.
6. **Clock validity**: Every faction has a clock with a named goal, segment
   count (4/6/8), and fill condition.
7. **Three-faction coverage**: At least one faction each in good, bad, and ugly
   roles.
8. **Frontmatter consistency**: Preserve `beads_id` field.
9. **Preserve `## Brief` and `## Rounds of Feedback` sections**: If a top-level section titled exactly `## Brief` exists, preserve it byte-for-byte — the original brief stays the original brief; feedback is feedback, not a new brief. If `## Brief` is absent (empty-brief artifact), do not add a placeholder. If a top-level section titled exactly `## Rounds of Feedback` exists at the end of the artifact, preserve it byte-for-byte. For both sections: do not edit, reformat, reorder, summarize, move, or remove any content within them. Krust will append the new round entry after you return; do not add a round entry yourself.

Do NOT skip these checks. The structural integrity of the situation document depends on them.

After rewriting the file, signal completion:
```bash
if [ -n "$KRUST_BEADS_ID" ]; then
  bd update $KRUST_BEADS_ID --set-metadata='artifact_written=true'
  bd update $KRUST_BEADS_ID --set-metadata='skill_complete=true'
fi
```
