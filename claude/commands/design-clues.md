---
name: design-clues
description: Design clues that route between nodes in a scenario graph — bidirectional, with rule-based validation. Use when fleshing out clues for an existing node graph after /design-nodes or hand-authored boards.
user_invocable: true
triggers:
  - design clues
  - clue design
  - flesh out clues
  - clues for node
  - clue brainstorm
argument-hint: "[optional: target node, figjam URL, situation doc path, fiction doc path]"
---

# Design Clues

You are helping the GM design clues that route PCs between nodes in an existing scenario graph. Your job is iterative: gather context, focus on a node (or a node-pair), generate candidates with the rules embedded, validate each against those rules, and present for the GM to pick or refine.

**Arguments:** $ARGUMENTS

This skill assumes a node graph already exists (from `/design-nodes` or hand-authored on a figjam board). It does not design the graph itself. If the GM wants to add or restructure nodes, point them to `/design-nodes`.

---

## The Seven Clue Rules

These rules govern every clue you propose and every validation you run. **Do not break them.** They are the working canon refined through iterative scenario design — they sit on top of `guides/node-design.md`, not next to it.

1. **Specificity.** Every clue points to a *specific node*, not a generic destination. "Tholvald" is not a node; "the inn at room 6" is. Vague environmental traces ("11 people were here," "boot prints") don't route PCs anywhere.

2. **Geometry — inbound is the rule.** *Every non-entry node needs ≥3 inbound clues.* The "2 lateral + 1 vertical outbound" pattern at any source node is a *byproduct* of every destination needing 3 inbound. Designing means asking: *"what 3 clues route PCs to this node?"* — even when the GM frames it as outbound.

3. **Direction-compelling.** A clue at A pointing to B must compel PCs to *travel* to B. If B's content or B's key NPC is delivered at A, B is no longer a node. (E.g., putting Krellic-the-bronzesmith in the tavern collapses Artisan Row — PCs would just talk to him in the tavern.)

4. **Realistic hook.** Every clue must connect to the PCs' active investigation — there has to be an in-scenario reason to follow it. "There's a cool statue with a maker's mark" doesn't motivate; the PCs aren't hunting artisans. The hook is what makes a routed lead *land*.

5. **Surface variety.** Within a node, the outbound clues should come from *different surfaces / modes of player engagement*: testimony (NPC asked), overheard (ambient gossip), look (environmental fixture), handle (physical object), document (paper / log / register), found-on-body (where bodies exist). Three NPCs all gossiping is one surface; three modes opens different player styles into the investigation.

6. **Stand out.** Clues must visually grab the player when they're in the room. Subtle ambient detail and quiet bartender memory don't land — clues need to be sticky enough that PCs notice and engage on their own initiative.

7. **Faction byproducts.** Clues come from what NPCs / factions were actually doing — not placed for navigation. (Per `guides/node-design.md` §9.) If the trace doesn't have an in-fiction reason to be there, it's wrong. Ask: *what was this person doing at this place at that time, and what would they accidentally leave?*

---

## Step 1: Gather Context

Read whatever inputs are available. None are strictly required, but the more you have, the better the candidates.

- **`guides/node-design.md`** — mandatory. The structural framework the Seven Rules rest on. Read it before generating anything.

- **Situation doc** — provides factions, NPCs, locations, current state of the scenario world. Path defaults to `<world>/<season>/situation.md` if not specified in `$ARGUMENTS`.

- **Fiction doc** — actor stories / pre-graph fiction sketch (e.g., `<world>/<season>/node-workspace/step-1-fiction-state.md`). **This is the gold mine for rule 7 (faction byproducts).** Each NPC's lead-up section names what they were doing, where, with whom — directly fueling specific candidate clues.

- **Figjam (the existing node graph)** — visualizes structure, layer geometry, and any clues already locked. To read:
  1. Extract `fileKey` and `nodeId` from a `figma.com/board/...` URL (board format = FigJam).
  2. Load tool schema: `ToolSearch` with `select:mcp__plugin_figma_figma__get_figjam`.
  3. Call `mcp__plugin_figma_figma__get_figjam` with the `fileKey` + `nodeId`.
  4. Parse: shape-with-text nodes (boxes), connectors (arrows — labels = clues), text blocks (DESIGN NOTES often live here).

If any input is missing, ask the GM where it lives or proceed with what you have. Do NOT invent context to fill the gap.

---

## Step 2: Pick a Focus

Ask the GM (or infer from `$ARGUMENTS`) **which node** is the focus and **which direction** they're thinking. Three modes:

- **Outbound mode** — "what 3 clues lead OUT of node X?" GM thinks from-source. Identify the destination nodes connected to X (sibling laterals + vertical), design one clue per destination.
- **Inbound mode** — "what 3 clues lead IN to node Y?" GM thinks toward-target. Identify the source nodes feeding Y, design one clue per source.
- **Edge mode** — "design the specific clue at A pointing to B." Single-edge focus, useful for fixing one weak link.

The skill works fluidly in any mode — the underlying invariant is rule 2 (every non-entry node needs ≥3 inbound), but the framing follows the GM's mental model.

Surface what's already locked at the focus node:
- Existing clues at this node (pointing where, on what surface)
- Existing clues to this node (from where, on what surface)
- Surface types already used (so you don't propose a duplicate per rule 5)

---

## Step 3: Generate Candidates

For each clue slot in the focus, generate **2-3 candidate clues** so the GM has options.

For each candidate:

1. **Look at the fiction first** (rule 7). Who was at the source node, what were they doing, and what would they accidentally leave behind that points to the target? Use the fiction doc and situation doc as substrate. Cite the in-fiction action.

2. **Pick a surface** that's distinct from existing surfaces at the source node (rule 5). Surfaces: testimony / overheard / look / handle / document / found-on-body.

3. **Make it stand out** (rule 6). Specific, vivid, hard to miss when PCs are in the room. Avoid background ambience.

4. **Make it node-specific** (rule 1). The clue must name or strongly imply the *specific* destination, not "somewhere in [city]."

5. **Make it direction-compelling** (rule 3). The target's key NPC or content stays at the target. The source carries the *trace*, not the substance.

6. **Hook it** (rule 4). Why would the PCs care about this clue *given what they're already investigating?* If the PCs have no active thread that intersects this candidate, redesign or surface that the hook is missing.

Present candidates with surface labels and in-fiction origin notes — the GM should be able to see *why* each candidate works (or might not).

---

## Step 4: Validate

For each candidate clue, run the seven rules as an explicit checklist. Show this validation table to the GM as part of the presentation. **Do not skip.**

| # | Rule | Question | Result |
|---|---|---|---|
| 1 | Specificity | Does the clue point to a specific node, not a vague region? | ✓ / ✗ / note |
| 2 | Geometry | After this clue, does the destination still get ≥3 inbound? | ✓ / ✗ / note |
| 3 | Direction | Is the target's content delivered AT the source? (Should be NO.) | ✓ / ✗ / note |
| 4 | Hook | Does the clue tie to an active PC investigation thread? | ✓ / ✗ / note |
| 5 | Surface variety | Is this surface distinct from others at this node? | ✓ / ✗ / note |
| 6 | Stand out | Is the clue visually / narratively sticky? | ✓ / ✗ / note |
| 7 | Faction byproduct | Does the clue have a cited in-fiction NPC origin? | ✓ / ✗ / note |

**Any FAIL means revise or reject.** **Any NOTE (partial concern) means surface it to the GM** — they may accept the trade-off or push back.

If a candidate fails rule 3 (direction-compelling), it usually needs the target's content moved off-source — e.g., replace the NPC themselves with a *trace of* the NPC. If it fails rule 4 (hook), check if the PCs have an existing thread to chase — if not, the clue may need to wait on additional fiction or a different surface.

---

## Step 5: Present and Iterate

For each clue slot, present validated candidates as:

```markdown
### Clue: {Source Node} → {Target Node}

**Candidate A** ({surface type})
{Vivid, specific, stand-out clue description.}

*In-fiction origin*: {NPC action that produced this trace, citing the fiction doc.}
*Hook*: {The active PC thread this connects to.}
*Validation*: {7/7 ✓, or list any notes / partial concerns.}

**Candidate B** ({different surface type})
{...}

**Candidate C** ({different surface type — optional third})
{...}
```

Then invite the GM to pick, refine, or redirect. The skill is **conversational** — once the GM chooses (or asks for more / different / better), continue:

- **Pick** → lock the candidate, move to the next slot.
- **Refine** → regenerate with the GM's added constraints.
- **Redirect** → new focus node or new direction; restart Step 2 with the new focus.

When all clue slots at the current focus are filled, ask if the GM wants to move to another node.

If the GM wants to **persist** the locked clues, ask where: a notes file in the workspace, the figjam board (point them to `/lk-board`), or just hold them in conversation.

---

## Constraints

- **Never invent campaign lore.** If situation / fiction / figjam don't ground a candidate, mark `[NEEDS GM INPUT]`.
- **Never collapse a destination node** by delivering its content at the source (rule 3).
- **Never propose a clue without a faction-byproduct origin** (rule 7). If you can't trace it to an NPC's actual action in the fiction, redesign.
- **Never propose three clues at one node from the same surface** (rule 5). Vary the modes.
- **Never assume the user thinks in inbound terms** even though that's the structural rule. They often think outbound; that's fine — translate internally without forcing the framing on them.
- **Never drift into scenario design.** This skill works WITHIN an existing graph. If the GM wants to add nodes or restructure tiers, redirect to `/design-nodes`.
- **Always show the validation table** — the GM has explicitly asked for visible rule-checking. Do not collapse it into implicit "trust me" generation.
- **Always cite the in-fiction origin** for every candidate. If you cannot, the candidate violates rule 7.
