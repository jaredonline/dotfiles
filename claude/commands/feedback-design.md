---
name: feedback-design
description: Apply feedback to a design document preserving structural invariants
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

1. **Section completeness**: Keep every section from the `/design` template — Problem, Constraints, Architecture, Interfaces, Data Flow, Key Decisions, Invariants, Open Questions, Tracking, Next Step. Do not drop sections even if feedback doesn't touch them.
2. **Interface specs are full**: every interface has a signature with types, parameters, and error cases. Prose-only interfaces are a regression.
3. **Key Decisions table**: every decision row still has a rejected alternative and a reason.
4. **Architecture diagram**: the Mermaid diagram matches the components described in the text. If feedback added or removed a component, update the diagram too.
5. **Invariants are concrete**: no vague goals — each invariant names a constraint the implementation must maintain.
6. **Tracking preserved**: `## Tracking` section still lists the Beads task ID. Do not rewrite the task ID.
7. **Frontmatter consistency**: preserve `beads_id` and any other frontmatter fields untouched.

Do NOT skip these checks — they're the structural contract between /design and /implement.

After rewriting the file, signal completion:
```bash
if [ -n "$KRUST_BEADS_ID" ]; then
  bd update $KRUST_BEADS_ID --set-metadata='artifact_written=true'
  bd update $KRUST_BEADS_ID --set-metadata='skill_complete=true'
fi
```
