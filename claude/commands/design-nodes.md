---
name: design-nodes
description: Design a node-based scenario graph with clues, revelations, and proactive timelines using Alexandrian node design. Use when the GM has a scenario premise and needs a playable investigation or exploration structure.
user_invocable: true
triggers:
  - design nodes
  - node graph
  - node design
  - scenario graph
  - investigation design
argument-hint: "[scenario brief, e.g. 'Investigate the disappearances in Thornwall' — optionally include a situation doc or known nodes]"
---

# Design Nodes

You are designing a node-based scenario graph for a TTRPG using a story-first flow. Your job is structural design of an investigation or exploration as a *snapshot of the world at scenario open*: a layered graph of self-contained situations, with clues that fall out of NPC behavior rather than being invented as breadcrumbs. The graph captures current state — who is where, what they are doing, what evidence sits where — not a forward plan or session script. You produce a complete, playable scenario document.

**Arguments:** $ARGUMENTS

**Input format:**
- **Scenario brief** (required) — what the scenario is about, what the PCs are walking into
- **Situation doc** (optional) — output from `/design-situation`, provides factions, NPCs, locations, and tensions to draw from
- **Known nodes** (optional) — specific locations, NPCs, or events the GM wants included as nodes
- **--redesign <path>** (optional) — path to a prior nodes doc for revision. The brief describes what to CHANGE, not the full scenario from scratch.
- **--speculate** (optional) — permits explicitly-marked forward speculation beyond the immediate-arc window. Off by default.

---

## Design Stance: State, Not Future

The node graph is a snapshot of the world at the moment the scenario opens. The situation doc locks in what has happened — events, motives, capabilities, current positions — and the node graph captures the *current state* of that situation: where the evidence sits, who is doing what, what the immediate trajectory is if no one intervenes.

The graph is **not a campaign plan, a session script, or a forecast**. Forward content is bounded to three kinds:

1. **NPC proactive behavior** in the immediate-future window (hours, days — the scope of the current arc) if PCs do not intervene.
2. **Default timeline**: the chronological version of the same — what unfolds across the arc if no one disrupts.
3. **Explicit speculation** when the GM passes `--speculate` (or the brief directs it). Default: off.

Anything else — multi-arc futures, PC-decision-tree branching, "if the PCs do X then Y" hypotheticals — is **forbidden by default**.

**Iteration model**: after each session, the GM revises the situation doc to lock in what changed. Then the node graph is regenerated against the revised situation. The graph is a *projection of the situation*, not a separate document with its own forward content.

This stance applies to every agent prompt in the flow.

---

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1a. Local reads | No — main agent | Brief, situation, override chain, world guide, node-design.md |
| 1b. /explore-rpg | No — single subagent | Unconditional campaign-context fetch |
| 2. Design layered graph (default 1-3-3-1) | No — main agent | Lock structure before fleshing |
| 3. Identify actors + actor-to-nodes map | No — main agent | Drives Wave A spawning |
| Wave A: Story Generators | Yes — 1 agent per actor | Independent NPC perspectives |
| Wave A-Review: Story Devil's Advocate | No — 1 agent | Story coherence check |
| Wave B: Node Designers | Yes — 1 agent per node | Receives filtered actor stories |
| 4. Assemble node doc | No — main agent | Weave stories + nodes; build Reverse Story / timeline / revelation list / map / audit |
| Wave C: Clean-Room Reviewer | No — 1 agent | Doc-only context, internal consistency |
| Wave D: Lore-Check | No — 1 agent | Full context final pass |
| 5. Final output | No — main agent | write artifact + `bd close` |

---

## Step 1a: Local Reads

Create and claim a bd task for this design:

```bash
bd_id=$(bd create "Nodes: [scenario brief]" --json | jq -r .id)
bd update "$bd_id" --claim
```

Capture the task ID into `$bd_id` for subsequent `bd update --notes` calls.

Read inputs in this order:

- **Brief.** Parse `$ARGUMENTS` for scenario brief, situation doc, and known nodes.

- **Situation doc.** Default `<world>/<season>/situation.md` if not specified.

- **Override chain** (most specific wins, missing files skipped):
  1. `<world>/<season>/{PHILOSOPHY,THEMES,GOALS}.md`
  2. `<world>/{PHILOSOPHY,THEMES,GOALS}.md`
  3. `guides/gm-standards.md`

  These govern tone and design philosophy for this world/season.

- **World guide** at `guides/<world>.md` if available.

- **`guides/node-design.md`** (mandatory — the structural reference the rest of this skill leans on).

The default output path is `$COCKPIT_DIR/state/nodes/<slug>.md`, where `<slug>` is the kebab-case scenario identifier.

If `--redesign <path>` is provided, read the file at that path and use it as the starting point. Parse the prior doc's layered graph, actor list, and node IDs as the baseline. The brief describes changes to make, not the full scenario. Preserve existing node IDs for unchanged nodes.

---

## Step 1b: /explore-rpg (Unconditional)

Invoke `/explore-rpg` with a context-gathering brief derived from the situation doc. The brief asks for:

- LK pages for every NPC named as an actor or supporting role in the situation doc
- LK pages for every faction and organization in the situation
- LK pages for every location the scenario touches
- Calendar / timeline context if the scenario is date-anchored
- Draw Steel mechanics relevant to encounters in the scenario (if any)

`/explore-rpg` returns a structured campaign-context bundle. Hold it in main-agent memory. It is filtered downstream:

- **Wave A (Story Generators)**: filtered per-actor — each Story Generator receives the LK material relevant to *its* actor (their NPC page, their faction's page, locations they frequent).
- **Wave B (Node Designers)**: filtered per-node — each Node Designer receives LK material relevant to NPCs and locations at *its* node.
- **Wave A-Review (Story Devil's Advocate)**: receives the full bundle for cross-reference of stated capabilities and goals.
- **Wave D (Lore-Check)**: receives the full bundle as canonical reference.
- **NOT passed to Wave C (Clean-Room)**: by design — clean-room has zero outside context.

This step is **unconditional**. Skipping because "the situation doc looks complete" is forbidden — the story-first design depends on richer context than the situation doc reliably carries.

The campaign-context bundle is transient main-agent state. It is not part of the final node doc output.

---

## Step 2: Design the Layered Graph

Lock the structure before any node fleshing. Read `guides/node-design.md` if you have not already.

### Default geometry (1-3-3-1)

Use this unless justified otherwise:

```
Layer 1: 1 node    — entry point
Layer 2: 3 nodes   — first investigation tier (all-to-all within layer)
Layer 3: 3 nodes   — second investigation tier (all-to-all within layer)
Layer 4: 1 node    — exit / climactic node

Connections:
  Layer 1 → all 3 nodes in Layer 2 (entry fans out)
  Layer 2 ↔ Layer 2 (all pairs connected within layer)
  Layer 2 → Layer 3 (each Layer 2 node has ≥1 elevator clue to Layer 3)
  Layer 3 ↔ Layer 3 (all pairs connected within layer)
  Layer 3 → Layer 4 (all 3 nodes converge to exit)
```

This shape satisfies the structural invariant (≥3 in, ≥3 out) for every middle-layer node geometrically:
- Each Layer 2 node has: 2 same-layer connections + ≥1 to Layer 3 + 1 from Layer 1 = ≥4 connections.
- Each Layer 3 node has: 2 same-layer connections + ≥1 from Layer 2 + 1 to Layer 4 = ≥4 connections.

**Entry node (Layer 1)** is exempted from the ≥3 inbound rule. Still needs ≥3 outbound (one per Layer 2 node, at minimum).
**Exit node (Layer 4)** is exempted from the ≥3 outbound rule. Still needs ≥3 inbound (one from each Layer 3 node).

### Node ID convention

Every node receives a unique ID in the format:

```
S<Season>.<Location>.<Layer>.<Number>
```

- **Season**: campaign season number (e.g. `S1`, `S3`).
- **Location**: 2-3 letter mnemonic for the geographic location (town, landmark, region). Established mnemonics are reused; new locations get new mnemonics that don't collide with existing ones.
- **Layer**: the layer this node sits in (`1`, `2`, `3`, `4` for the default 1-3-3-1 shape).
- **Number**: sequential within `<Location>.<Layer>`, starting at `1`.

Examples:
- `S1.CD.1.1` — Season 1, Caer-Dineval, Layer 1 (entry), Node 1
- `S3.TV.2.1` — Season 3, Tholvald, Layer 2, Node 1
- `S3.TV.2.2` — Season 3, Tholvald, Layer 2, Node 2
- `S3.DR.4.1` — Season 3, Durdrak, Layer 4 (exit), Node 1

Encoding the layer in the ID makes the graph's structure legible at a glance — a reader sees `S3.TV.2.1 → S3.DR.3.2` and immediately knows that's an inter-tier elevator clue from layer 2 to layer 3. IDs are stable once assigned. On `--redesign`, check existing IDs to avoid collisions.

### Modifications to the default

Modifications to 1-3-3-1 are allowed but require written justification per `guides/node-design.md`:

- **Linear chokepoint pair** (e.g. inciting incident + immediate aftermath, where Layer 1 → Layer 2 wants to be Layer 1A → Layer 1B → Layer 2): document the chokepoint as intentional.
- **2-node tier**: each node owes its 3rd clue to a different tier.
- **4+ node tier**: each node still needs ≥3 in / ≥3 out, which now requires explicit clue counting.
- **Mentor / proactive ally side-nodes** (like B6 Veranthia in Durdrak v6): can hang off the layered structure as side-nodes that strengthen but do not gate the main flow.

The agent commits to the layered structure **before** any node fleshing happens. Output of Step 2 is: layer count, node IDs, planned inter-node connections, one-line situation sketches per node.

---

## Step 3: Identify Actors and Build the Actor-to-Nodes Map

An **actor** is an NPC (or a coherent acting group of NPCs) with independent goals and arc-spanning movements. Examples from Durdrak v6:

| Actor | Type | Independent goals? | Arc-spanning? |
|---|---|---|---|
| Renaith / her Crew | Group | Yes | Yes |
| The Fey Agent | NPC | Yes | Yes |
| Theryn'dros (as Veranthia) | NPC | Yes | Yes |
| Marevek | NPC | Yes (coerced) | Yes |
| Halvrek | NPC | Yes (narrowed) | Yes |
| Tessyn | NPC | Yes | Yes |
| Brelka the chandler | Minor NPC | No (background) | No |

Actors get story agents. Minor NPCs (Brelka, Torben, Vinnik) are drawn into actors' stories as supporting cast, not given their own.

**Cosmic forces are not actors.** A Primordial, a curse, an ambient threat — these are forces, not actors. They appear in actor stories (as influence on Halvrek, on Margaret's drift) but do not receive a Story Generator. The actor-identification rule: an actor must be capable of *intentional action*.

For each actor, identify **which nodes the actor passes through, influences, or leaves evidence at**. This produces an actor-to-nodes map:

```json
{
  "Renaith": ["S3.TV.2.1", "S3.TV.2.2", "S3.DR.4.1"],
  "Fey Agent": ["S3.TV.2.2", "S3.DR.4.1"],
  "Marevek": ["S3.CD.1.1", "S3.DR.3.1"],
  "Halvrek": ["S3.CD.1.1", "S3.DR.3.2"]
}
```

The map drives Wave B (each Node Designer receives the stories of actors who pass through *its* node).

---

## Wave A: Story Generators (Parallel)

One agent per actor. **Spawn ALL Wave A agents in ONE message.** Each agent receives this prompt VERBATIM, with bracketed fields filled in:

```
You are a story generator for one NPC actor in a TTRPG scenario.

## Your Actor
{Name, role, faction, key goals from situation doc}

## The Layered Node Graph
{Layer structure, all nodes with one-line situation sketches, the actor-to-nodes
 map showing which nodes YOUR actor moves through}

## The Situation
{Full situation doc}

## World/Season Context
{PHILOSOPHY, THEMES, GOALS content}

## Task

Tell the story of how YOUR actor moved through the scenario *from their
perspective*. Not omniscient. Not narrated by the GM. The story should:

1. **Start before the scenario opens.** Where was the actor when this began?
   What were they doing? What were they thinking?

2. **Walk through every node the actor touched**, in chronological order
   (which is rarely the order the PCs will visit them). For each:
   - What did the actor do here?
   - What did they want?
   - What did they leave behind, on purpose or by accident?
   - Who did they interact with?

3. **End at the actor's current state** as the scenario opens. What is the actor
   doing right now? What is their immediate next planned action (the one already
   in motion at scenario open)? Stop there.

4. **Note inadvertent traces**: things the actor did NOT mean to leave behind,
   but did (a footprint, a witness who saw something, a habit they don't know
   they have, a payment that left a record). These are gold for the Node Designer
   step — they're where clues come from.

5. **Stay in character.** The actor's perspective is limited. They don't know
   what other actors are doing unless they have direct evidence. They don't know
   the cosmic stakes unless their goals reach there. Write what THEY know.

## Constraints
- Do NOT design clues yet. Describe behavior. Clues are the next agent's job.
- **Do NOT speculate forward.** The story captures the actor's state up to and
  including the moment the scenario opens, plus the immediate next action
  already in motion. Do NOT describe what the actor will do next session, what
  PCs might force them to do, or future-arc consequences. The graph is a state
  snapshot, not a campaign plan. (See Design Stance.)
- Do NOT plot what PCs will do. Describe what the actor does if PCs never appear.
- Stay grounded in the situation doc. Do not invent new factions, motivations,
  or capabilities.
- Be specific. "Met with the broker" is bad. "Met with the broker in the back
  room of the Spar and Splinter, paid in chits, never gave a name, lit a candle
  and put it out twice while talking" is good.

## Output Format
Return a structured story with:
- **Pre-scenario state**: 1-2 paragraphs
- **Movement through nodes** (in chronological order, one section per node):
  - Node ID + name
  - What they did
  - What they wanted
  - What they left behind (intentional)
  - What they left behind (inadvertent)
  - Witnesses (who saw or heard something)
- **Current state at scenario open**: 1 paragraph
```

The output of Wave A is N actor stories, where N is the number of actors.

---

## Wave A-Review: Story Devil's Advocate

After Wave A returns, **one** Devil's Advocate agent reviews all collected stories for per-actor coherence. Send this prompt VERBATIM:

```
You are reviewing a set of NPC stories for a TTRPG scenario. Each story
describes how one actor moved through the scenario from their own perspective.
Your job is to check, for each story, whether the NPC's behavior is coherent
with their stated goals, capabilities, and the world they live in.

## Stories
{All N actor stories from Wave A, with each story labeled by actor name}

## The Situation
{Full situation doc — for cross-reference of stated goals and capabilities}

## Task

For each actor's story, ask:

1. **Goal coherence**: do the actions in this story serve the actor's stated
   goals? Are there moves the actor makes that don't connect to anything they
   want?

2. **Capability plausibility**: does the actor have the means to do what the
   story says they did? (Resources, knowledge, access, time, allies.)

3. **Internal consistency**: do later actions in the story contradict earlier
   ones? Does the actor learn something at one point and then act as if they
   don't know it later?

4. **Inadvertent omniscience**: does the actor know things they shouldn't —
   information that would only be available to another actor or the GM?

5. **Missing motivation**: are there moments where the story relies on
   "they decided to" without the deciding making sense?

6. **Forward-speculation creep**: does the story include forecasts about
   future sessions, branching PC outcomes, or future-arc consequences? (See
   the Design Stance — these should be flagged as out-of-scope unless the
   brief explicitly directed them.)

## Output Format

For each actor whose story has issues, return:
- **Actor**: name
- **Issue**: one sentence
- **Severity**: high (story is incoherent), medium (logic gap), low (polish)
- **Suggested fix**: one sentence

If a story is coherent, do not list it. Brevity is correct.

If all stories are coherent, say so and stop.
```

After return, the main agent adjudicates each finding (**ACCEPT** apply, **REJECT** with reason, **MODIFY** with different fix) and edits the affected stories *before passing them to Wave B*. Adjudications are recorded in the doc's `## Story Review` section:

```markdown
## Story Review

| # | Severity | Actor | Issue | Decision | Notes |
|---|----------|-------|-------|----------|-------|
| 1 | medium | Renaith | Story has her in two cities on the same day | MODIFY | Compressed timeline; she sent a courier, didn't go in person |
| 2 | low | Veranthia | Forward speculation about season-4 confrontation | ACCEPT | Removed; not directed by brief |
```

---

## Wave B: Node Designers (Parallel)

Wave B receives the *adjudicated* actor stories from Wave A. The main agent filters them per node before spawning Wave B agents.

One agent per node. **Spawn ALL Wave B agents in ONE message.** Each agent receives this prompt VERBATIM:

```
You are a node designer for one node in a TTRPG scenario graph.

## Your Node
{Node ID, name, type (Location/Person/Organization/Event/Activity), one-line
 situation sketch from Step 2}

## The Layered Node Graph
{Full layer structure, all nodes, the planned inter-node connections from Step 2}

## Actor Stories Relevant to Your Node
{Filtered: ONLY the stories of actors who passed through your node, drawn from
 the actor-to-nodes map. Each story is included in full so you can see what the
 actor was doing before and after their visit.}

## The Situation
{Full situation doc — you can reference it for tone, NPC voices, etc.}

## Task

Design this node as a self-contained SITUATION (not a scene — a toolkit, not a
script). Specifically:

1. **Situation**: What's happening here right now, in present tense. Concrete
   details: who's here, what they're doing, physical environment.

2. **Key NPCs**: Who's here. For each: name, goal, disposition toward PCs, what
   they know, what they share vs. hide.

3. **Outgoing clues** (leads to other nodes per the planned layered graph):
   Each clue must derive from one of the actor stories above. Cite the actor
   and the action whose consequence this clue is.
     - Example: "Clue: a tally-stick in Torben's pouch with 4 notches —
       derived from Renaith's story (she paid muscle on day-rate starting four
       days before the ambush)."
   The minimum is ≥3 outbound clues for non-exit nodes; ≥1 elevator clue per
   layer up.

4. **Clues pointing HERE**: Cross-reference to other nodes' outbound clues that
   target this node. (You'll list these; they're a consistency check, not new
   content.)

5. **Proactive behavior**: What this node does on its own timeline if PCs
   don't visit, **bounded to the arc's immediate-future window** (the next
   hours, days, or weeks of the active scenario — not future arcs). Use trigger
   conditions: "If [condition], then [action]." Should derive from actor stories.

6. **Push/Pull**: Pull (why PCs would seek this) + Push (what forces them here).

## Constraints
- Every clue must trace to an actor's action in the stories above. If a clue
  cannot be grounded that way, do not include it.
- Vary discovery methods: physical evidence, testimony, document, observed
  event, item found on a body, dropped artifact, overheard, archive entry.
- Clues that ONLY exist if a specific PC outcome occurs (e.g. "Torben tells
  them...") must have a physical-evidence equivalent at the same node.
  Interrogation accelerates the investigation; it must not gate it.
- The situation is a toolkit, not a script. Do not write what PCs will do.
- **The graph captures the world's current state, not a future plan.** Do not
  speculate beyond the arc's immediate-future window. Do not branch on
  hypothetical PC choices. (See Design Stance.)

## Output Format
Return the node section in the standard format (see `guides/node-design.md`
worked example or the current design-nodes Output Format template).
```

**Implementation note (empty-actor nodes):** For the rare node where no actor went (e.g., a static archive of static lore), the Node Designer receives an empty filtered-stories input and must derive clues from the situation doc directly.

The output of Wave B is M node sections, where M is the total node count (default 8).

---

## Step 4: Assemble the Node Doc

The main agent weaves the actor stories and node sections into the standard output document. Specifically:

1. **Mermaid node-map diagram** generated from the layered graph (nodes as boxes shaped/colored by type, edges labeled with clue type, entry/exit nodes marked distinctly).

2. **Revelation List** built from accumulated evidence across nodes — what PCs can learn, and which nodes contain the evidence.

3. **Default Timeline** built by interleaving the actor stories chronologically (timestamps or trigger conditions, every proactive node represented).

4. **Reverse Story** generated **FROM the actor stories** — not retrofitted from clues. Each actor's chronological story becomes a contribution to the Reverse Story timeline; each clue's provenance traces to a specific story action. This is the inverse of the old "Reverse Story as audit" rule: stories are the source, clues are the derivative.

5. **Reachability Audit** by counting connections per node — entry/exit identified, ≥2 independent paths confirmed for non-entry nodes, chokepoints/dead ends flagged with rationale, structural pattern confirmed.

---

## Wave C: Clean-Room Reviewer

One agent, spawned with **only the assembled node doc** as context. No situation doc, no guides, no campaign history, no LK MCP access. Send this prompt VERBATIM:

```
You are reviewing a TTRPG scenario node graph document for internal consistency
and logical plausibility. You have NO context beyond this document. You do not
know the world's lore, the situation's full background, or the PCs.

## Document
{The full assembled node doc}

## Task

Read the document. For each of these questions, identify any places where the
answer is "no" or "unclear":

1. **Does the node graph make structural sense?**
   - Are nodes reachable as the doc claims?
   - Do clue connections actually appear in both directions?
   - Are the layers identifiable and disciplined?

2. **Are NPCs acting with intent and goals in mind?**
   - Does each NPC's behavior in the Reverse Story make sense given their stated
     goals?
   - Are there moments where an NPC does something that feels out of character
     based on what's elsewhere in the doc?
   - Are there inexplicable actions where motivation is hand-waved?

3. **Are the clues plausible consequences?**
   - Could each clue actually have been left by the action that allegedly
     produced it?
   - Are any clues "planted" — i.e. they only exist because the GM needs them
     to exist?
   - Are any clues so convenient that they break suspension of disbelief?

4. **Is there unnecessary indirection or whimsy?**
   - Does any section spend words on flavor that doesn't pay off?
   - Are there nodes whose purpose isn't clear?
   - Are there NPCs who don't earn their presence?
   - Are there subplots that feel grafted on rather than emergent?

5. **Is the proactive behavior coherent?**
   - Do NPC timelines make sense in their own terms?
   - Are trigger conditions grounded in stated goals?

6. **Is the doc a state snapshot, not a forward plan?**
   - Does any section speculate about future sessions, multi-arc consequences,
     or PC-decision-tree branches?
   - Is forward content limited to NPC proactive behavior in the arc's
     immediate window and the default timeline?
   - Flag any forward speculation that doesn't carry an explicit "directed by
     brief" marker.

## Output Format
Return a numbered list of recommendations. For each:
- **Severity**: high (graph is broken / NPC is incoherent), medium (logic gap
  but recoverable), low (nice-to-have polish)
- **Where**: section / node / NPC affected
- **What's wrong**: one sentence
- **Suggested fix**: one sentence

If the document is sound, say so and stop. Do not invent problems.
```

After Wave C returns, the main agent adjudicates each recommendation as **ACCEPT** (apply the suggested fix), **REJECT** (write a one-sentence reason), or **MODIFY** (apply a different fix). Decisions are recorded in a `## Clean-Room Review` section appended to the doc:

```markdown
## Clean-Room Review

| # | Severity | Where | Issue | Decision | Notes |
|---|----------|-------|-------|----------|-------|
| 1 | medium | B2 | Renaith's payment trail uses chits no one in the doc has explained | MODIFY | Added a B3 clue that frames "strange currency" as a Tessyn observation |
| 2 | low | A1 | Margaret's PIP scene runs long for a single beat | REJECT | Multi-PC scene; needed for character establishment |
| 3 | high | C1 | Volescu's cooperation is implausible given threat level | ACCEPT | Added paragraph on his cutout status and pre-paid fee |
```

---

## Wave D: Lore-Check

After clean-room decisions are applied, run the existing `/lore-check` skill against the revised doc. The lore-check agent has full context (situation doc, LK MCP, world guide, the campaign-context bundle from Step 1b).

Lore-check produces the standard `## Lore Check` section appended to the doc. If no conflicts found, include the section with "No lore conflicts detected."

---

## Step 5: Final Output

1. Write the artifact and commit it:

   ```bash
   slug="thornwall-disappearances"        # kebab-case scenario identifier
   out="$COCKPIT_DIR/state/nodes/$slug.md"

   # If the slug collides with an existing file, bump it (-v2, -v3, ...).
   if [ -e "$out" ]; then
     n=2
     while [ -e "$COCKPIT_DIR/state/nodes/$slug-v$n.md" ]; do n=$((n+1)); done
     slug="$slug-v$n"
     out="$COCKPIT_DIR/state/nodes/$slug.md"
   fi

   # ... write the assembled node doc to "$out" ...

   git -C "$COCKPIT_DIR" add "$out"
   git -C "$COCKPIT_DIR" commit -m "Add node graph: $slug"
   git -C "$COCKPIT_DIR" push
   ```

2. Close the bd task:

   ```bash
   bd close "$bd_id"
   ```

---

## Framework Reference

Node-based scenario design theory — the inverted Three Clue Rule, node types, clue types, push/pull navigation, structural patterns, the reachability invariant, dead-end rules, situations-vs-scenes — lives in **`guides/node-design.md`** in the world repo. That guide is the authoritative source for every structural rule referenced in this skill. Do not duplicate it here; do not work from a half-remembered version of it.

If `guides/node-design.md` does not exist in the project being worked on, fall back to the Alexandrian's *Node-Based Scenario Design* Parts 1-9 at thealexandrian.net.

The single most important rule: **every non-entry node has ≥3 outbound leads and ≥3 inbound leads, and every node is reachable via ≥2 independent paths.** The default **1-3-3-1** layered shape (1 entry → 3-node tier → 3-node tier → 1 exit, all-to-all within tiers, ≥1 elevator clue per layer up) makes this geometrically natural. Modifications require written justification per `guides/node-design.md`.

---

## Output Format

```markdown
# Node Graph: {Scenario Name}

## Premise
{What's happening. What PCs are walking into. What they don't know yet.}

## Node Map
{Mermaid diagram: nodes as boxes (colored/shaped by type), edges as clue connections.
Labeled edges show clue type. Entry point(s) marked distinctly.}

## Revelation List
| # | Revelation | Type | Nodes Containing Evidence |
|---|-----------|------|--------------------------|
| R1 | {what PCs can learn} | lead / evidence / both | {which nodes} |

## Nodes

### S<Season>.<Location>.<Layer>.<Number> — {Node Name} — {Type}

**Situation**: {Present tense. Concrete details.}

**Key NPCs**: {Who's here, what they want, how they react.}

**Clues pointing OUT**:
1. → {Node ID}: {Clue} — {discovery method} — *derived from {Actor}'s story ({action})*

**Clues pointing HERE**:
- From {Node ID}: {What clue points here}

**Proactive behavior**: {Trigger condition. What changes if PCs delay. Bounded to arc's immediate-future window.}

**Push/Pull**:
- Pull: {why PCs seek this}
- Push: {what forces PCs here}

## Reverse Story

{1-2 sentence summary: what happened and why — the actions that produced the evidence players find. Built from the actor stories upstream.}

1. **{Actor}** {action} -> Produces: {Clue X} at **{Node ID}**
2. **{Actor}** {action} -> Produces: {Clue A} at **{Node ID}**, {Clue C} at **{Node ID}**
3. **{Actor}** inadvertently {mistake} -> Produces: {Clue E} at **{Node ID}**
...

## Default Timeline
| When | Event | Triggered By | Affects Nodes |
|------|-------|-------------|---------------|

## Story Review

| # | Severity | Actor | Issue | Decision | Notes |
|---|----------|-------|-------|----------|-------|

## Clean-Room Review

| # | Severity | Where | Issue | Decision | Notes |
|---|----------|-------|-------|----------|-------|

## Lore Check
{From /lore-check, or "No lore conflicts detected."}

## Reachability Audit
**Entry points**: {which nodes, why}
**Chokepoints**: {single-entry nodes, rationale}
**Dead ends**: {nodes with no outgoing clues, intentional?}
**Coverage**: Every non-entry node reachable via ≥2 paths: {YES/NO}
**Structural pattern**: {1-3-3-1 default, or modification with justification}

## Brief

> {brief, verbatim, each line prefixed with `> `}
```

The `## Brief` section reproduces the scenario brief verbatim as a blockquote. If the brief is empty, omit the section entirely.

---

## Validation Checklist

**Structural invariant (from `guides/node-design.md`):**
- [ ] Default 1-3-3-1 shape used unless modification is justified in writing
- [ ] Every non-entry node has ≥3 outbound leads
- [ ] Every non-entry node has ≥3 inbound leads
- [ ] Every non-entry node is reachable via ≥2 independent clue paths
- [ ] Within each tier, every node connects to every other node in the tier
- [ ] Every node in tier N has ≥1 outbound lead (elevator clue) to a node in tier N+1
- [ ] Chokepoint nodes are flagged as intentional with rationale
- [ ] Dead ends are flagged; bonus clues to dead ends are not counted toward 3-clue minimums
- [ ] Node IDs follow `S<Season>.<Location>.<Layer>.<Number>` and are stable across `--redesign`

**Story-first invariants:**
- [ ] Every clue traces to an actor's action in some story
- [ ] Reverse Story generated from actor stories (not retrofitted from clues)
- [ ] Wave B agents only received stories of actors who pass through their node
- [ ] No actor's story contains forward speculation beyond the immediate-arc window (unless `--speculate` was passed)
- [ ] Cosmic forces appear as influence in actor stories, not as their own Story Generator outputs

**Scenario completeness:**
- [ ] Revelation list covers both leads and evidence
- [ ] Default timeline shows what NPCs/factions do without PC intervention
- [ ] At least one node has proactive behavior with an explicit trigger condition
- [ ] Each node is a situation (toolkit), not a scene (script)
- [ ] Clue discovery methods are varied (physical evidence, testimony, document, observation, overheard, found-on-body, etc.)
- [ ] Forward content bounded: no multi-arc futures, no PC-decision branches (unless `--speculate`)

**Review sections present:**
- [ ] `## Story Review` section present with adjudications (ACCEPT/REJECT/MODIFY)
- [ ] `## Clean-Room Review` section present with adjudications
- [ ] `## Lore Check` section present

**Output:**
- [ ] Output file written to `$COCKPIT_DIR/state/nodes/<slug>.md` and committed+pushed

---

## Constraints

- **The graph captures world state at scenario open, not a forward plan.** Forward content is restricted to NPC proactive behavior in the arc's immediate window, the default timeline, and explicitly directed `--speculate` content.
- **`/explore-rpg` is unconditional in Step 1b.** Skipping it because the situation doc looks complete is forbidden.
- **Every parallel wave (A, B, C, D) is spawned in a single message.** Sequential spawning negates parallelism and is forbidden. (Wave A-Review, C, and D are single-agent waves.)
- **Wave B agents only receive stories of actors who pass through their node.** Filtering enforces local relevance and bounds context size.
- **Every clue traces to an actor action in some story.** This is the inverse of the old "Reverse Story as audit" rule: stories are the source, clues are the derivative.
- **Clean-room agent has zero outside context.** No situation doc, no guides, no MCP access, no campaign history. Only the assembled node doc.
- **Never invent campaign lore.** If the scenario brief doesn't specify something and it's not in the situation doc, override chain, world guide, or `/explore-rpg` bundle, use `[NEEDS GM INPUT]` placeholders.
- **Output sections are conditional.** Only include a section if it has substantive content — no empty placeholders.
- **Nodes are situations, not scenes.** Prep a toolkit (personnel, layout, information, NPC goals), not a script.
- **The Framework section governs all structural decisions.** Structural choices must be justified by `guides/node-design.md`.
