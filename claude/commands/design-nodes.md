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

You are designing a node-based scenario graph for a TTRPG. Your job is structural design of an investigation or exploration using node-based methodology — building a network of self-contained situations connected by discoverable clues, with redundant paths and proactive timelines. You produce a complete, playable scenario document.

**Arguments:** $ARGUMENTS

**Input format:**
- **Scenario brief** (required) — what the scenario is about, what the PCs are walking into
- **Situation doc** (optional) — output from `/design-situation`, provides factions, NPCs, locations, and tensions to draw from
- **Known nodes** (optional) — specific locations, NPCs, or events the GM wants included as nodes
- **--redesign <path>** (optional) — path to a prior nodes doc for revision. When provided, the brief describes what to CHANGE, not the full scenario from scratch.

---

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Parse input + gather context | No — main agent | Needs to identify scope and existing material |
| 2. Design node skeleton + clue network | No — main agent | Graph structure requires coherent design |
| 3. Flesh out node situations | Yes — 1 agent per node | Each gets full skeleton, details are independent |
| 4. Devil's advocate | No — 1 agent | Challenge assumptions about PC behavior using full campaign context |
| 5. Consistency check | No — 1 agent | Verify nodes match situation doc before synthesis |
| 6. Synthesize + validate | No — main agent | Weaves clues, builds timeline, checks reachability |
| 7. Output + validation | No — main agent | Formats final document |
| 8. Lore check | No — 1 agent | Needs completed content |

---

## Step 1: Parse Input + Gather Context

Run:

```bash
bd_id=$(krust bd-start task "Nodes: [scenario brief]")
```

`krust bd-start` auto-detects mode: under krust (KRUST_BEADS_ID set) it prints the existing task ID; standalone it creates and claims a new task with project-resolved labels. Capture the output into `$bd_id` for subsequent `bd update --notes` calls.

Read inputs:
- Under krust, the scenario brief, situation doc, known nodes, campaign_dir, and actions_dir live in bd metadata: `bd show $bd_id --json`, then extract from `.metadata.krust` (`brief`, `campaign_dir`, `inputs.situation`, `inputs.nodes`, `actions_dir`). The output path is `$KRUST_OUT`.
- Standalone, parse `$ARGUMENTS` for scenario brief, situation doc, and known nodes.

Identify the **world** and **season** the scenario belongs to. Read PHILOSOPHY.md, THEMES.md, and GOALS.md from the override chain (most specific wins):
1. `<world>/<season>/{PHILOSOPHY,THEMES,GOALS}.md`
2. `<world>/{PHILOSOPHY,THEMES,GOALS}.md`
3. `guides/gm-standards.md` (global, always available)

Missing files are skipped. These govern tone and design philosophy for this world/season — apply them throughout node design (clue tone, NPC voices, what kinds of revelations matter). Also read the world guide at `guides/<world>.md` if available.

**Always read `guides/node-design.md` before Step 2.** It is the authoritative reference for every structural rule the skill enforces.

Standalone, the default situation doc is `<world>/<season>/situation.md` and the default output path is `<world>/<season>/nodes/<arc-slug>.md`.

If a situation doc is provided, extract factions, NPCs, locations, and tensions as candidate nodes. If campaign context is needed and LegendKeeper is available, query via the explore-rpg pattern.

If `--redesign <path>` is provided, read the file at that path and use it as the starting point. Parse the prior doc's nodes, clue network, timeline, and revelation list as the baseline. The brief describes changes to make, not the full scenario.

---

## Step 2: Design Node Skeleton + Clue Network

**Read `guides/node-design.md` before this step.** It is the authoritative reference for every structural rule below. The summary: ≥3 outbound and ≥3 inbound clues per node; default to layer-cake structure; the geometry does the work.

Design the structural graph before fleshing out details. Work in this order — do not skip to clues before tiers are placed.

### Step 2a: Identify tiers

Decompose the scenario into **tiers** (layers in the layer cake). A tier is a phase of investigation or action — e.g. *setup → investigation → conclusion*, or *surface → middle → deep*, or *introduction → escalation → climax*. Most arc-scale scenarios have 3-4 tiers. Scenarios with more than 5 tiers usually want to be split into multiple arcs.

Name each tier with one phrase that describes its function in the scenario.

### Step 2b: Place nodes per tier

**Default: 3 nodes per tier.** This is not arbitrary — with 3 nodes per tier, each node naturally satisfies the structural invariant (≥3 outbound, ≥3 inbound clues) via 2 within-tier connections + ≥1 inter-tier connection.

Wider or narrower tiers require **written justification** in the design doc:
- **2-node tier**: each node owes its 3rd clue to a different tier or a sub-node. Acceptable when the design demands a tight chokepoint pair (e.g. inciting incident + immediate aftermath). Document the justification.
- **4+ node tier**: each node still needs ≥3 outbound and ≥3 inbound clues, which now requires explicit clue-counting and is harder to keep coherent. Acceptable for genuinely fan-shaped investigation (the dragnet, the urban canvass) where multiple independent leads converge. Document the justification.

For each node: name, type (Location / Person / Organization / Event / Activity), one-line situation sketch.

### Step 2c: Draw within-tier connections

Within each tier, **every node connects to every other node in the tier**. For a 3-node tier, this is 3 edges. Each edge represents at least one concrete clue (physical evidence, testimony, document, observable event) that an antagonist or faction's actions have produced. Clues are consequences, not breadcrumbs.

### Step 2d: Draw inter-tier connections

Each node in tier N has **≥1 outbound clue to a node in tier N+1** (the "elevator clue"). For a 3-node tier feeding a 3-node tier, this is 3 edges minimum. More is fine; wider fan-out makes the next tier reachable from more directions.

Inter-tier movement is bidirectional in the layer cake — PCs may go forward, backward, or sideways. Plan for backward references (a tier-2 node confirms what was suspected at tier-1) and lateral discovery (a tier-3 node reveals why a tier-1 thread mattered).

### Step 2e: Identify entry points

Mark where PCs start. The entry tier is usually tier 1, but may include proactive nodes that reach into PCs' lives from later tiers (a villain sends a messenger; a faction summons them).

### Step 2f: Verify the structural invariant

For each node, count:
- **Outbound clues** (leads to other nodes): ≥3
- **Inbound clues** (leads from other nodes): ≥3 for non-entry nodes
- **Independent paths**: ≥2 (the reachability invariant — survives one accident)

Flag any node that fails this check. Either add clues or merge/remove the node. Do not proceed to Step 3 with a graph that fails the invariant.

### Step 2g: Flag chokepoints, dead ends, and proactive nodes

- **Chokepoints**: nodes reachable by only one path. Intentional (e.g. inciting incident, final confrontation) is fine — label as intentional. Accidental is a bug — fix it.
- **Dead ends**: nodes with no outbound clues. Acceptable as red herrings or cleared locations. Clues *to* dead ends are bonus and don't count toward the 3-outbound minimum at the source node.
- **Proactive nodes**: at least one. These act on a default timeline whether PCs visit or not. Trigger conditions stated explicitly.

### Step 2h: Build the revelation list

What PCs can learn across the scenario, and which nodes contain the evidence. This becomes a section in the final document.

---

## Step 3: Flesh Out Node Situations (Parallel)

Spawn one agent per node using the Agent tool (model=opus). Each agent receives the full skeleton so it can write clues that connect coherently to the rest of the graph.

**Agent prompt** (send this VERBATIM to each node agent, filling in the bracketed fields):

```
You are fleshing out one node in a TTRPG scenario graph.

## Your Node
{node name, type, and brief situation sketch from lead}

## Full Node Skeleton
{all nodes, their types, and planned clue connections — the complete graph structure}

## Scenario Premise
{what the scenario is about}

## World/Season Context
{PHILOSOPHY, THEMES, GOALS content from the override chain — apply this to tone, NPC voices, clue style}

## Task
Flesh out this node as a self-contained SITUATION (not a scene — a toolkit, not a script):

1. **Situation** — What's happening here, right now, in present tense.
   Concrete details: who's here, what they're doing, physical environment.
   Enough that the GM can adjudicate ANY player approach, not just the expected one.

2. **Key NPCs** — Who's here. For each:
   - Name, goal, disposition toward PCs
   - What they know (and what they'll share vs. hide)
   - How they react if PCs are friendly / hostile / sneaky

3. **Outgoing clues** (leads to other nodes per the skeleton):
   - Make each clue CONCRETE and discoverable
   - Vary discovery methods: physical evidence, testimony, documents,
     observable events, things overheard, items found on a body
   - Each clue should feel natural to the situation, not planted

4. **Proactive behavior** — What this node does on its own timeline:
   - Explicit trigger: "If [condition], then [action]"
   - What changes if PCs delay visiting
   - Does this node seek out PCs? Send messages? Create events?

5. **Push/Pull**:
   - Pull: what's desirable here (treasure, information, allies, answers)
   - Push: what forces PCs here (threats, ambushes, summons, explosions)

Return the completed node section in the output format below.
```

**Spawn ALL node agents in ONE message.** Do not serialize them.

---

## Step 4: Devil's Advocate

After the node agents return their drafts, spawn a Devil's Advocate agent that stress-tests the graph from the players' perspective. This agent needs the full campaign context — not just the scenario brief — because players draw on everything they've experienced.

**Devil's Advocate** (Agent, model=opus):

```
You are a Devil's Advocate stress-testing a TTRPG scenario node graph. Your job is to think like PLAYERS, not like the GM who designed this. Players don't follow logical paths — they fixate on details the GM considers throwaway, they remember NPCs from 10 sessions ago, they pursue personal grudges, and they try solutions the designer never imagined.

## Campaign Context
{use explore-rpg to gather full campaign context: session history, active PCs, NPC relationships, faction states, unresolved hooks, player tendencies}

## Drafted Nodes
{paste all node outputs from Step 3}

## Scenario Premise
{the scenario brief}

## Task

1. **Gather campaign context.** Use the LegendKeeper MCP (search_content, list_worlds) and any available session history to build a picture of:
   - What the PCs have done so far
   - Which NPCs they have strong relationships with (positive or negative)
   - Unresolved plot threads or grudges
   - Past player behavior patterns (do they negotiate? kick down doors? split the party?)

2. **Challenge every assumption about PC behavior.** For each node, ask:
   - What if the PCs skip this entirely? What breaks?
   - What if the PCs attack/befriend/interrogate someone the design assumes they'll ignore?
   - What if the PCs fixate on a detail here and refuse to move on?
   - What if they bring an NPC from a previous arc into this situation?
   - What if they try to solve this with resources or allies not accounted for in the graph?

3. **Identify missing nodes.** Based on campaign context, are there locations, NPCs, or factions the players are LIKELY to seek out that aren't in the graph? These aren't random — they're places/people the players already know about and would logically turn to.

4. **Flag fragile assumptions.** Where does the graph assume players will:
   - Follow clues in a particular order?
   - Trust an NPC they have no reason to trust?
   - Ignore an obvious lead because it's not "in the design"?
   - Not use an ability, spell, or resource that trivially bypasses a node?

## Rules
- Ground every challenge in SPECIFIC campaign history or player behavior, not hypotheticals.
- "Players might do anything" is not useful. "The party has allied with the Thornwall militia twice before and will likely go to them first" IS useful.
- Don't redesign the graph. Flag gaps and suggest where new nodes or clue paths might be needed.
- If the graph is robust to unexpected player behavior, say so and stop.

## Output Format
### Missing Nodes
| # | Suggested Node | Type | Why Players Will Seek This | Campaign Evidence |
|---|---------------|------|---------------------------|-------------------|
| 1 | {name} | {type} | {why} | {specific session/NPC/event reference} |

### Fragile Assumptions
| # | Node | Assumption | Why It's Fragile | Suggested Fix |
|---|------|-----------|-----------------|---------------|
| 1 | {node} | {what the design assumes} | {why players won't do this} | {brief suggestion} |

### Stress Test Summary
{2-3 sentences: how robust is this graph to real player behavior?}
```

After the agent returns:
- **Missing nodes:** Add any that the main agent agrees are well-grounded (backed by campaign evidence, not speculation). Add them to the node skeleton and spawn additional node agents for them.
- **Fragile assumptions:** Address high-impact ones by adding redundant clue paths or adjusting node situations. Flag the rest for GM review in the final output.
- If the agent identifies new nodes to add, flesh them out (spawn node agents) before proceeding.

---

## Step 5: Consistency Check (if situation doc provided)

If a situation doc was supplied as input, spawn a consistency-check agent to verify the drafted nodes are faithful to it. Skip this step if no situation doc was provided.

**Consistency Checker** (Agent, model=opus):

```
You are checking a set of drafted TTRPG scenario nodes for consistency against the situation doc they were derived from.

## Situation Doc
{paste the full situation doc}

## Drafted Nodes
{paste all node outputs from Step 3}

## Task
Compare the drafted nodes against the situation doc and report inconsistencies. Check:

1. **NPCs** — Every NPC referenced in the nodes should match the situation doc's characterization (goals, allegiances, disposition). Flag NPCs that contradict the situation doc or appear in nodes but not in the situation doc (invented without basis).

2. **Factions** — Faction goals, resources, and relationships in the nodes should align with the situation doc. Flag any faction behavior in the nodes that contradicts stated faction goals or capabilities.

3. **Locations** — Locations used as nodes should be consistent with geographic or spatial details in the situation doc. Flag invented locations that contradict established geography.

4. **Tensions & Conflicts** — The central tensions driving the scenario should match the situation doc's conflict structure. Flag nodes that introduce conflicts not grounded in the situation doc's faction dynamics.

5. **Timeline & Causality** — Proactive behaviors and trigger conditions in the nodes should be consistent with faction goals and capabilities described in the situation doc. Flag timeline events that require capabilities or motivations not established in the source material.

## Rules
- Only flag genuine inconsistencies — differences that would confuse or contradict the situation doc.
- New detail that EXTENDS the situation doc without contradicting it is fine. Don't flag enrichment.
- For each inconsistency, quote the relevant line from the situation doc and the conflicting node content.
- If no inconsistencies found, say so and stop.

## Output Format
### Inconsistencies
| # | Node | Element | Node Says | Situation Doc Says | Severity |
|---|------|---------|-----------|-------------------|----------|
| 1 | {node} | {npc/faction/etc} | {what the node claims} | {what the doc says} | high/medium/low |

### Summary
{one-line: consistent / N inconsistencies found}
```

After the agent returns:
- **If inconsistencies found:** fix them in the drafted nodes before proceeding to Step 5. High-severity inconsistencies must be resolved; medium/low can be flagged for GM review.
- **If consistent:** proceed to Step 6.

---

## Step 6: Synthesize + Validate

After all node agents return, weave the results into a coherent document.

1. **Ensure clue consistency:** Node A's outgoing clue to Node B must match Node B's incoming clue list. Fix mismatches.

2. **Build the revelation list** from actual clues in the fleshed-out nodes — not from the skeleton. Update it with concrete evidence references.

3. **Write the default timeline:** what happens if PCs don't intervene. Use explicit timestamps or trigger conditions. Every proactive node should appear here.

4. **Run the reachability audit:**
   - Entry points identified
   - Every non-entry node reachable via at least 2 independent paths
   - Chokepoints flagged with rationale
   - Dead ends flagged as intentional or not
   - Structural pattern confirmed

5. **Write the Reverse Story.** Walk through the events chronologically — what actions by antagonists, factions, or other forces produced the evidence players find. Each numbered action must produce at least one clue in the graph. Include mistakes, oversights, and unintended consequences — these often produce the most discoverable clues. The story doesn't need to focus on one character; it just needs to be a logical, narrative explanation of how the clues came to be. Use the format from the Output Format section.

6. **Orphan check.** List every clue from all nodes. Verify each appears in the Reverse Story action sequence. Flag orphans — clues with no in-world origin. Either ground them in an action or flag for GM review.

---

## Step 7: Output + Validation

Run the validation checklist. Remove any empty sections. Output the final document in the format below.

**File naming:** When running under krust ($KRUST_OUT is set), write the output file to `$KRUST_OUT`. Do not use a user-suggested filename — the harness requires this exact path for completion detection. If the user's prompt suggests a different filename, use `$KRUST_OUT` anyway and note the user's preferred name in the document title.

---

## Step 8: Lore Check

Run the `/lore-check` skill against the complete node graph document. It will cross-reference all proper nouns, dates, relationships, and faction names against canonical LegendKeeper data.

Append the lore check results as a `## Lore Check` section at the end of the output document (before ## Reachability Audit). If no conflicts found, include the section with "No lore conflicts detected."

---

## Step 9: Signal Completion

1. Hand the artifact to krust:

   ```bash
   krust artifact nodes <slug> <path-you-wrote-to>
   ```

   `<slug>` is the kebab-case scenario identifier (e.g. `thornwall-disappearances`). Under krust the wrapper uses the slug to canonicalize the artifact path; standalone it writes to `$COCKPIT_DIR/state/nodes/<slug>.md` and commits+pushes directly.

2. Append a line to the campaign INDEX.md via an action (no subcommand yet — this stays in its current form):

   ```bash
   echo '{"type": "index_append", "path": "<campaign_dir>/designs/INDEX.md", "line": "- [<slug>](<slug>.md) — <title>"}' > "$ACTIONS_DIR/index-append.json"
   ```

   `<campaign_dir>` and `<title>` come from bd metadata (read earlier in Step 1).

3. Close the bd task:

   ```bash
   krust bd-finish "$bd_id"
   ```

   Under krust this is a no-op (the wrapper closes the task on approval); standalone it closes the task.

---

## Framework Reference

Node-based scenario design theory — the inverted Three Clue Rule, node types, clue types, push/pull navigation, structural patterns, the reachability invariant, dead-end rules, situations-vs-scenes — lives in **`guides/node-design.md`** in the world repo. That guide is the authoritative source for every structural rule referenced in this skill. Do not duplicate it here; do not work from a half-remembered version of it.

If `guides/node-design.md` does not exist in the project being worked on, fall back to the Alexandrian's *Node-Based Scenario Design* Parts 1-9 at thealexandrian.net.

The single most important rule to keep in front of you while designing: **every non-entry node has ≥3 outbound leads and ≥3 inbound leads, and every node is reachable via ≥2 independent paths.** The default layer-cake structure (3 nodes per tier, all-to-all within-tier, ≥1 inter-tier elevator clue per node) makes this geometrically natural. Wider or narrower tiers require written justification per Step 2b.

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

### {Node Name} — {Type}

**Situation**: {Present tense. Concrete details.}

**Key NPCs**: {Who's here, what they want, how they react.}

**Clues pointing OUT**:
1. → {Node X}: {Clue} — {discovery method}

**Clues pointing HERE**:
- From {Node A}: {What clue points here}

**Proactive behavior**: {Trigger condition. What changes if PCs delay.}

**Push/Pull**:
- Pull: {why PCs seek this}
- Push: {what forces PCs here}

## Reverse Story

{1-2 sentence summary: what happened and why — the actions that produced the evidence players find.}

1. **{Actor}** {action} -> Produces: {Clue X} at **{Node Y}**
2. **{Actor}** {action} -> Produces: {Clue A} at **{Node B}**, {Clue C} at **{Node D}**
3. **{Actor}** inadvertently {mistake} -> Produces: {Clue E} at **{Node F}**
...

**Orphaned clues**: {list of clues with no provenance, or "None"}

## Default Timeline
| When | Event | Triggered By | Affects Nodes |
|------|-------|-------------|---------------|

## Reachability Audit
**Entry points**: {which nodes, why}
**Chokepoints**: {single-entry nodes, rationale}
**Dead ends**: {nodes with no outgoing clues, intentional?}
**Coverage**: Every non-entry node reachable via ≥2 paths: {YES/NO}
**Structural pattern**: {type and why}
```

---

## Validation Checklist

**Structural invariant (the rules that govern; from `guides/node-design.md`):**
- [ ] Every non-entry node has ≥3 outbound leads
- [ ] Every non-entry node has ≥3 inbound leads
- [ ] Every non-entry node is reachable via ≥2 independent clue paths
- [ ] Each tier has 3 nodes by default; wider or narrower tiers are justified in writing
- [ ] Within each tier, every node connects to every other node in the tier
- [ ] Every node in tier N has ≥1 outbound lead to a node in tier N+1
- [ ] Chokepoint nodes (single-path entry) are flagged as intentional with rationale
- [ ] Dead ends are flagged; bonus clues to dead ends are not counted toward the 3-clue minimums

**Scenario completeness:**
- [ ] Revelation list covers both leads and evidence
- [ ] Default timeline shows what NPCs/factions do without PC intervention
- [ ] At least one node has proactive behavior with an explicit trigger condition
- [ ] Each node is a situation (toolkit), not a scene (script)
- [ ] Clue discovery methods are varied (physical evidence, testimony, document, observation, overheard, found-on-body, etc.)
- [ ] Every clue in the graph traces to an action in the Reverse Story
- [ ] Every section present is substantive (no empty placeholders)

**Output:**
- [ ] Output file written to `$KRUST_OUT` (when running under krust)

---

## Constraints

- **Never invent campaign lore.** If the scenario brief doesn't specify something and it's not in the context bundle, use `[NEEDS GM INPUT]` placeholders.
- **Output sections are conditional.** Only include a section if it has substantive content — no empty placeholders.
- **Nodes are situations, not scenes.** Prep a toolkit (personnel, layout, information, NPC goals), not a script.
- **The Framework section governs all creative decisions.** Structural choices must be justified by the theory above.
