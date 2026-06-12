---
name: feedback-situation
description: Apply feedback to a situation design document preserving structural invariants
user_invocable: false
---

Read the situation document at `<world>/<season>/situation.md`.

The feedback text comes from `$ARGUMENTS`. If it is empty, the user edited the file by hand — no feedback text to apply, review their edits for consistency only.

---

Create and claim a bd task for this feedback round:
```bash
bd_id=$(bd create "Feedback: situation" --json | jq -r '.id')
bd update "$bd_id" --claim
```

Read the situation document.

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
8. **Frontmatter consistency**: Preserve any existing frontmatter fields; do not
   strip or reorder them.
9. **Preserve `## Brief` and manage `## Rounds of Feedback`**: If a top-level
   section titled exactly `## Brief` exists, preserve it byte-for-byte — the
   original brief stays the original brief; feedback is feedback, not a new
   brief. If `## Brief` is absent (empty-brief artifact), do not add a
   placeholder. Preserve any existing `## Rounds of Feedback` entries
   byte-for-byte — do not edit, reformat, reorder, summarize, move, or remove
   prior round content. When you have applied feedback text, you own the round
   log: append a new round entry to `## Rounds of Feedback` (creating the
   section at the footer if it does not yet exist). The footer ordering is
   `## Open Threads` → `## Brief` → `## Rounds of Feedback`. Each round entry
   records the round number and the verbatim feedback applied, e.g.:

   ```markdown
   ## Rounds of Feedback

   ### Round 1
   > {feedback text, verbatim, each line prefixed with `> `}
   ```

   If the feedback text was empty (hand edit), do not add a round entry.

Do NOT skip these checks. The structural integrity of the situation document depends on them.

After rewriting the file, commit and push it, then close the bd task:
```bash
git add <world>/<season>/situation.md
git commit -m "Apply feedback to situation"
git push
bd close "$bd_id"
```
