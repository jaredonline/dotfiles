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

**When running under krust** ($KRUST_BEADS_ID is set):

Read inputs from the beads task metadata:
- Run: `bd show $KRUST_BEADS_ID --json`
- Extract from `.metadata.krust`:
  - `brief` — the scenario brief
  - `campaign_dir` — campaign directory for context
  - `inputs.situation` — path to situation doc (if any)
  - `inputs.nodes` — known nodes (if any)
  - `actions_dir` — directory for action JSON files
- The output path is available directly as `$KRUST_OUT`
- Do NOT create a new beads task — the wrapper already created one

**When running standalone** ($KRUST_BEADS_ID is not set):

Parse `$ARGUMENTS` for scenario brief, situation doc, and known nodes.

- If a situation doc is provided, extract factions, NPCs, locations, and tensions as candidate nodes
- If campaign context is needed and LegendKeeper is available, query via the explore-rpg pattern (parallel LK + system MCP queries)
- If $KRUST_BEADS_ID is not set, create a beads task to track this work:

```bash
bd create --title="Nodes: {scenario brief}" --type=task --json
```

Claim the task: `bd update <task-id> --claim`

---

## Step 2: Design Node Skeleton + Clue Network

Design the structural graph before fleshing out details.

1. **Identify 5-9 nodes.** For each: name, type (Location / Person / Organization / Event / Activity), one-line situation sketch.

2. **Choose a structural pattern:**
   - Cloud (default) — dense, asymmetric, modular
   - Conclusions — multiple paths funnel toward a climactic node
   - Layers — nodes in tiers, each tier points to the next deeper
   - Dual Track — two independent clusters of 4-6 nodes each
   - Hybrid — combine patterns as needed

3. **Sketch clue connections** between nodes — which nodes point to which, and roughly what kind of clue (physical evidence, testimony, document, observation, etc.). Clues should be consequences of antagonist/faction actions, not planted breadcrumbs.

4. **Identify entry points** — where PCs start. Mark these distinctly.

5. **Verify the Reachability Invariant:** every non-entry node must be reachable via at least 2 independent clue paths. This is a structural minimum, not a target.

6. **Flag chokepoints** — nodes reachable by only one path. Intentional chokepoints are fine (e.g., a final confrontation); accidental ones are bugs. Flag all with rationale.

7. **Build the revelation list** — what PCs can learn across the scenario, and which nodes contain the evidence.

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

After writing the artifact file:
```bash
if [ -n "$KRUST_BEADS_ID" ]; then
  bd update $KRUST_BEADS_ID --set-metadata='artifact_written=true'
fi
```

---

## Step 8: Lore Check

Run the `/lore-check` skill against the complete node graph document. It will cross-reference all proper nouns, dates, relationships, and faction names against canonical LegendKeeper data.

Append the lore check results as a `## Lore Check` section at the end of the output document (before ## Reachability Audit). If no conflicts found, include the section with "No lore conflicts detected."

After validation and lore check complete:
```bash
if [ -n "$KRUST_BEADS_ID" ]; then
  bd update $KRUST_BEADS_ID --set-metadata='validated=true'
fi
```

---

## Step 9: Signal Completion

If running under krust, signal completion:

1. If you wrote to `$KRUST_OUT`: no artifact action needed.
   If you wrote to a different path: emit artifact action:
   ```bash
   echo '{"type": "artifact", "source": "<path-you-actually-wrote>"}' > "$ACTIONS_DIR/artifact.json"
   ```

2. Write index append action (unchanged):
   ```bash
   echo '{"type": "index_append", "path": "<campaign_dir>/designs/INDEX.md", "line": "- [<slug>](<slug>.md) — <title>"}' > "$ACTIONS_DIR/index-append.json"
   ```

3. Signal completion:
   ```bash
   bd update $KRUST_BEADS_ID --set-metadata='actions_emitted=true'
   bd update $KRUST_BEADS_ID --set-metadata='skill_complete=true'
   ```

Where `<campaign_dir>`, `<slug>`, and `<title>` are the values read from beads metadata earlier.

---

## Framework: Node-Based Scenario Design Theory

This section governs all structural decisions in node graph design.

### The Problem with Plots

Most RPG adventures are linear: A -> B -> C -> D. Each transition is a chokepoint — if players don't find the clue at A pointing to B, the adventure breaks. The Three Clue Rule helps but doesn't fix the structure. Branching paths require 5x the prep for the same play time.

### The Core Insight: The Inverted Three Clue Rule

The original Three Clue Rule says: for any conclusion you want PCs to reach, include at least 3 clues.

The inversion says: if PCs have access to any 3 clues, they will reach at least 1 conclusion.

Scatter clues across nodes pointing to different destinations. Same prep effort as linear design, vastly more flexibility.

### What Is a Node?

A node is a point of interest — a self-contained situation. Five types:

1. **Location** — A physical place
2. **Person** — A specific individual
3. **Organization** — A group
4. **Event** — A time-bound occurrence
5. **Activity** — A task PCs perform

Nodes can nest: fractal principle — zoom in to find sub-nodes, zoom out to find it's part of a larger node.

### Clue Types

Two types:

- **Leads** (scenario-solve clues) — Point to new nodes. Drive navigation.
- **Evidence** (concept-solve clues) — Point to solutions/conclusions. Help PCs understand what happened.

Leads and evidence often overlap. But evidence alone does not satisfy the Inverted Three Clue Rule unless it also functions as a lead.

### Navigation: Push and Pull

- **Pull** — PCs seek a node because it's desirable
- **Push** — PCs are forced into a node

Seven navigation methods:

1. **Clues** — primary method
2. **Geography** — physical adjacency
3. **Temporality** — time-triggered events
4. **Random triggering** — encounter tables, use sparingly
5. **Proactive nodes** — content that seeks out PCs (most powerful)
6. **Following trails** — player-initiated, GM-facilitated
7. **Player initiative** — PCs investigate something unplanned

### Proactive Nodes and Default Timelines

Proactive nodes don't wait for PCs — they act on their own timeline. Every proactive node should have an explicit trigger condition. The default timeline is the backbone — write what every NPC/faction does if PCs never intervene.

### Structural Patterns

- **The Cloud** — dense, asymmetric, modular (default target)
- **Conclusions** — multiple paths funnel toward a climactic node
- **Layers** — nodes in tiers, each tier points to the next deeper
- **Dual Tracks** — two independent clusters of 4-6 nodes each (most useful for campaigns)
- **Dead Ends** — acceptable, naturally absorbed by redundant clue structure

### Cognitive Limits

Working memory holds 5-9 items. Keep active node clusters in this range. Dual-track is designed around this.

### The Reachability Invariant

Every non-entry node must be reachable via at least 2 independent clue paths. This is the Inverted Three Clue Rule as a structural invariant. It's a minimum, not a target. Nodes reachable by only one path are chokepoints — flag for GM review.

### Situations, Not Scenes

Every node is a situation (toolkit: personnel, layout, information, NPC goals) not a scene (script). Prep enough to adjudicate any player approach.

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

- [ ] Every non-entry node is reachable via at least 2 independent clue paths
- [ ] No node has zero outgoing clues unless it's a deliberate dead end or resolution node
- [ ] Chokepoint nodes are flagged with rationale
- [ ] Revelation list covers both leads and evidence
- [ ] Default timeline shows what NPCs/factions do without PC intervention
- [ ] At least one node has proactive behavior
- [ ] Node count is 5-9; larger scenarios split into clusters
- [ ] Each node is a situation (toolkit), not a scene (script)
- [ ] Clue discovery methods are varied
- [ ] Every clue in the graph traces to an action in the Reverse Story
- [ ] Every section present is substantive
- [ ] Output file written to `$KRUST_OUT` (when running under krust)

---

## Constraints

- **Never invent campaign lore.** If the scenario brief doesn't specify something and it's not in the context bundle, use `[NEEDS GM INPUT]` placeholders.
- **Output sections are conditional.** Only include a section if it has substantive content — no empty placeholders.
- **Nodes are situations, not scenes.** Prep a toolkit (personnel, layout, information, NPC goals), not a script.
- **The Framework section governs all creative decisions.** Structural choices must be justified by the theory above.
