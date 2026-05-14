---
name: design-situation
description: Design a political situation with factions, NPCs, goals, and conflict using the Three Faction Rule and proactive roleplaying frameworks. Use when the GM has a conflict premise and needs a playable faction landscape.
user_invocable: true
triggers:
  - design situation
  - faction design
  - political situation
  - three faction
  - situation design
argument-hint: "[conflict premise, e.g. 'Hobgoblins threaten the duchy' — optionally add PC goals and setting context]"
---

# Design Situation

You are designing a political situation for a TTRPG scenario using rigorous situation design frameworks. Your job is creative design grounded in the Three Faction Rule and Proactive Roleplaying — not freeform brainstorming. Every faction, NPC, and goal must serve the central conflict and create pressure on the PCs.

**Arguments:** $ARGUMENTS

The input format:
- **Conflict premise** (required) — the core tension driving the situation
- **PC goals** (optional) — what the player characters are trying to accomplish
- **Setting context** (optional) — campaign, location, era, system
- **Existing situation doc** (optional) — output from a prior run, provided for revision
- **--redesign <path>** (optional) — path to a prior situation doc for revision. When provided, the brief describes what to CHANGE, not the full situation from scratch.

---

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Parse input + gather context | No — main agent | Needs to identify campaign, check LK |
| 2. Define conflict + faction sketches | No — main agent | Inter-faction tension requires coherent design |
| 3. Flesh out factions | Yes — 1 agent per faction | Each gets all sketches for coherence, details are independent |
| 4. Synthesize + timeline | No — main agent | Weaves factions into tension, assigns NPCs |
| 5. Output + validation | No — main agent | Formats final document |
| 6. Lore check | No — 1 agent | Needs completed content |

---

## Step 1: Parse Input + Gather Context

- Identify the **world** and **season** the situation belongs to. Read PHILOSOPHY.md, THEMES.md, and GOALS.md from the override chain (most specific wins):
  1. `<world>/<season>/{PHILOSOPHY,THEMES,GOALS}.md`
  2. `<world>/{PHILOSOPHY,THEMES,GOALS}.md`
  3. `guides/gm-standards.md` (global, always available)

  Missing files are skipped. These files govern tone, themes, and design philosophy for this world/season — apply them throughout faction and NPC design.
- Read the world guide from `guides/<world>.md` if available.
- If campaign context is needed and LK is available, query via explore-rpg pattern (parallel MCP queries)
- If an existing situation doc is provided, parse it as the starting point for revision
- If `--redesign <path>` is provided, read the file and use it as the starting point
- Parse the prior doc's factions, NPCs, goals, and timeline as the baseline
- The brief describes changes to make, not the full situation

Run:
```bash
bd_id=$(krust bd-start task "Situation: [conflict premise]")
```

`krust bd-start` auto-detects mode: under krust (KRUST_BEADS_ID set) it prints the existing task ID; standalone it creates and claims a new task with project-resolved labels. Capture the output into `$bd_id` for subsequent `bd update --notes` calls.

Read inputs:
- Under krust, the conflict premise lives in bd metadata: `bd show $bd_id --json`, extract from `.metadata.krust.brief` (e.g. `bd show $bd_id --json | jq -r '.metadata.krust.brief // empty'`).
- Standalone, parse `$ARGUMENTS` for conflict premise, PC goals, and setting context. The brief is `$ARGUMENTS`.
- If no PC goals are provided, continue designing the situation but generate `## PC-Facing Opportunities` instead of `## PC Goal Paths`. Use `[NEEDS GM INPUT]` only where a goal-specific answer depends on unknown campaign intent.

---

## Step 2: Define Conflict + Faction Sketches

- Identify the central conflict from the premise
- Sketch factions using the Three Faction Rule as a starting lens:
  - **Good**: On the back foot, struggling, this is why PCs matter. There may be multiple good-aligned factions with different approaches.
  - **Bad**: The threat, active, aggressive, pursuing goals that create the central conflict. Usually one primary antagonist faction, but rival bad factions or splinter threats are valid.
  - **Ugly**: The tipping point, doesn't fit neatly, complex — whoever wins this faction's support likely wins. Multiple ugly factions create richer political landscapes.
- The minimum is 3 factions; the maximum is whatever the scenario demands. Don't pad, but don't artificially compress either.
- For each faction sketch: name, role (good/bad/ugly), identity (one sentence), ideology (one sentence), relationship to other factions
- This is the ARCHITECTURE — it must be coherent before agents flesh out details

---

## Step 3: Flesh Out Factions (Parallel)

Spawn one agent per faction using the Agent tool (model=opus). Each agent receives:

```
You are fleshing out one faction for a TTRPG political situation.

## Your Faction
{faction name, role, identity sketch from lead}

## All Factions in This Situation
{full sketches of ALL factions — identities, roles, ideologies, how they relate to each other}

## Central Conflict
{the conflict driving the situation}

## World/Season Context
{PHILOSOPHY, THEMES, GOALS content from the override chain — apply this to tone, NPC voices, and ideology}

## PC Goals (if known)
{player character goals}

## Task
Flesh out this faction using the Proactive Roleplaying framework:

1. **Identity** — what they are, one sentence
2. **Area of Operation** — where they're active, where they might expand
3. **Power Level** — resources, military strength, political influence, magical capability
4. **Ideology** — core beliefs, worldview that binds members
5. **Methods** — how they pursue goals (diplomacy, violence, subterfuge, ritual, etc.)
6. **Goals at three timescales:**
   - Long-term: vague but directional (the true scope of ambition)
   - Mid-term: concrete stepping stones (generate arcs, spawn short-term goals)
   - Short-term: immediate actions (generate encounters) — 2-3 of these
7. **2-4 Key NPCs**, each categorized as ally/villain/patron relative to PCs:
   - Name, personal goal (DISTINCT from faction goal), personality (2-3 adjectives),
     method of pursuit, what they offer or threaten
   - Allies: goals align with PCs, equals, will leave when goals diverge
   - Villains: goals conflict with PCs, must be villainous + powerful + goal-driven
   - Patrons: grant faction resources in exchange for advancing patron goals, higher status
8. **A progress clock**: what they're working toward, 4/6/8 segments, what fills/empties it
9. **PC Leverage**:
   - What the PCs can offer this faction
   - What the PCs can threaten, expose, steal, protect, or sabotage
   - What information, access, alliance, or material resource matters to this faction
10. **Concessions**:
   - What the faction or its NPCs can give the PCs
   - What pressure or proof is required before they give it
   - What price, favor, risk, or compromise they demand
11. **Clock Interventions**:
   - What PC actions fill the clock
   - What PC actions stall or empty the clock
   - What PC actions redirect the clock into a different faction conflict

Design goals that CREATE TENSION with the other factions and with PC goals.
Short-term goals should produce concrete encounter hooks.
The "good" side should be struggling. The "ugly" faction should be a genuine tipping point.
Player-facing guidance must describe tools and pressure points the GM can adjudicate, not scripted PC plans. Do not write "if the PCs do X, then Y" branches.
If PC-specific leverage depends on unknown campaign facts, use `[NEEDS GM INPUT]`.

**FORMATTING RULE**: Each bold field (**Role in conflict**, **Identity**, **Area of Operation**, etc.) MUST be separated by a blank line so they render as distinct paragraphs in markdown. Without blank lines, CommonMark collapses them into a single unreadable paragraph.

Return the completed faction section in the output format below.
```

Spawn ALL faction agents in ONE message.

---

## Step 4: Synthesize + Timeline

- Weave faction agent results into a coherent document
- Build a faction relationship map (Mermaid diagram)
- Map PC goal intersections (table: PC Goal | Faction | Overlap | Tension)
- If PC goals were provided, synthesize `## PC Goal Paths` immediately after intersections. For every PC goal, include at least one concrete path with viable approaches, required leverage, key factions/NPCs, obstacles, costs/risks, progress signals, and clock effects.
- If PC goals were not provided, synthesize `## PC-Facing Opportunities` instead. These are likely playable opportunities implied by the premise, not invented player goals.
- Write the default timeline: what each faction does if PCs never intervene
- Identify internal divisions IF they serve this situation (apply three-faction lens recursively only where it adds pressure — this is a tool, not a mandate)
- Ensure NPC personal goals are distinct from faction goals and that every key NPC offers or threatens something useful at the table

---

## Step 5: Output + Validation

Run the validation checklist. Remove any empty sections. Output the final document.

Write the situation document to `$KRUST_OUT` (when set) or to `<world>/<season>/situation.md` (standalone). The `situation.md` is a living doc — if it already exists, this run produces a revision (use `--redesign <path>` to make that explicit).

Signal completion:
```bash
krust artifact situations <slug> <path-you-wrote-to>
krust bd-finish "$bd_id"
```

When running under krust ($KRUST_OUT is set), write the output file to `$KRUST_OUT`.

---

## Step 6: Lore Check

Run the `/lore-check` skill against the complete situation document. It will cross-reference all proper nouns, dates, relationships, and faction names against canonical LegendKeeper data.

Append the lore check results as a `## Lore Check` section at the end of the output document (before ## Open Threads). If no conflicts found, include the section with "No lore conflicts detected."

---

## Framework: Situation Design Theory

This section governs all creative decisions. Apply it precisely — do not improvise alternatives.

### Philosophy: Don't Prep Plots

Prep situations, not plots. A plot is a predetermined sequence of events — it's fragile, breaks when players deviate, and robs them of agency. A situation is a set of circumstances: who wants what, what resources they have, what they'll do if unopposed. The GM describes what exists and what NPCs are doing; the story emerges from player choices.

Prep tools, not contingencies. Don't plan "if PCs do X, NPCs do Y" — that's wasted work because PCs won't do X, or if they do, everything else changes too. Instead, prep the guard roster, the NPC's daily schedule, the faction's resources and goals. That's enough to adjudicate any player plan.

Player achievement guidance is still a tool, not a contingency. Give the GM leverage, obstacles, concessions, costs, signals, and clock effects so they can adjudicate many player plans. Do not write scripted solution branches or assume what the PCs will do.

Scenario timelines are the engine of a living world. Write what NPCs and factions do if PCs never intervene. When PCs disrupt the timeline, revise it based on NPC goals and resources. The timeline is a living document, not a script.

Multiple antagonists, protect none. Prep several villains. Don't predetermine who the "big bad" is. Whoever survives contact with the PCs becomes the major recurring threat organically. Let enemies flee when losing — those who escape become memorable.

### The Three Faction Rule

Every political situation needs at least three factions filling the roles of good, bad, and ugly — but a role can be filled by more than one faction.
- Good: On the back foot. Struggling. This is why PCs matter — the good side needs heroes. Multiple good factions may disagree on method while sharing cause.
- Bad: The threat. Active, aggressive, pursuing goals that create the central conflict. One primary antagonist usually drives the scenario, but rival threats or opportunistic predators add pressure.
- Ugly: The tipping point. Doesn't fit neatly into either side. Complex, messy, innately human. Multiple ugly factions create a richer political landscape where alliances shift.

This is a starting lens, not a straitjacket. Three is the minimum, not the target. Let the premise dictate how many factions it needs:

Six steps to a political landscape:
1. Conflict — Start simple. It doesn't need to be original.
2. Factions — At least three factions covering good, bad, and ugly roles. Give them evocative names. The ugly faction(s) are the tipping point.
3. Characters — Players interact with NPCs, not factions. Cast each faction with sharp, unmistakable personality. The good side should have relatively incompetent leadership.
4. Ideology — A worldview that binds faction members. Simple broad strokes — dead easy to understand.
5. Methods — How each faction pursues its goals. Methods determine what kind of encounters each faction generates.
6. Twist — Fractal nesting: subfactions (three within each), uber-factions (above the three), cross-faction alliances. Fractal depth is the GM's choice — go as deep as the scenario demands, no deeper.

Morally complex factions: Make every faction "bad" but put a "good" subfaction inside each. Or make all factions "ugly" with good and bad actors inside each.

Faction removal and addition: In long campaigns, players can destroy a faction. A new one fills the power vacuum. Player actions can fracture or merge factions.

### Proactive Roleplaying Faction Design

For each faction, establish these attributes in order:
1. Faction Identity — what they are, what they do. Use archetypes: Government, Labor, Crime, Religion.
2. Area of Operation — where active, where might expand. Location implies conflicts.
3. Power Level — military strength, wealth, political influence, magical capability.
4. Ideology — core principles. The most important attribute — drives goal design at all timescales.

### Goal Design

Goals are the engine of proactive play. Design them to collide with PC goals.
- Long-term: true scope of ambition, can be vague
- Mid-term: concrete stepping stones, generate arcs
- Short-term: immediate actions, generate encounters — build as-needed

### PC Goal Path Design

When PC goals are known, every goal needs concrete paths toward achievement. A path is not a plot; it is a set of playable handles the GM can adjudicate:
- **Viable approaches**: 2-4 broad methods such as negotiate, expose, raid, protect, recruit, investigate, bargain, disrupt, legitimize, or divide
- **Required leverage**: proof, alliance, access, resource, captive, secret, ritual component, legal claim, public support, or military position
- **Key factions/NPCs**: named actors from this situation whose cooperation, opposition, resources, or secrets matter
- **Obstacles**: concrete faction, NPC, resource, geography, timing, ideology, or information barriers
- **Costs/risks**: enemies made, clock acceleration, collateral damage, debt, reputation loss, moral compromise, or lost opportunities
- **Progress signals**: observable changes that tell the GM and players the goal is closer
- **Clock effects**: which clocks are filled, stalled, emptied, or redirected by plausible PC action

When PC goals are not known, design PC-facing opportunities instead: pressure points and openings implied by the situation. Mark goal-specific unknowns with `[NEEDS GM INPUT]` rather than inventing campaign intent.

### NPC Design in Proactive Fantasy

Three categories:
- Allies: goals align with PCs, equals not subordinates, less powerful than PCs
- Villains: goals conflict with PCs, must be villainous + powerful + goal-driven
- Patrons: grant resources in exchange for advancing patron goals, higher status

### Tracking: Clocks

Blades in the Dark-style progress clocks. 4/6/8 segments. Fill based on PC interactions.

For every faction clock, also identify how PC action can fill it, stall or empty it, or redirect it into conflict with another faction. These are adjudication tools, not promises that PCs will take those actions.

### Faction Tracking Shorthand

Five lines per faction: name, relative power, location, brief description, 1-2 long-term goals. Track no more than a dozen factions.

---

## Output Format

```markdown
# Situation: {Name}

## Central Conflict
{The core tension. One paragraph.}

## Factions

### {Faction Name}

**Role in conflict**: {Good / Bad / Ugly}

**Identity**: {What they are. One sentence.}

**Area of Operation**: {Where they're active.}

**Power Level**: {Resources, military strength, political influence.}

**Ideology**: {Core beliefs.}

**Methods**: {How they pursue goals.}

**Goals**:
- **Long-term**: {Vague but directional}
- **Mid-term**: {Concrete stepping stones}
- **Short-term**: {Immediate actions}

**Key NPCs**:
| Name | Role | Personal Goal | Personality | Method | Offers | Threatens |
|------|------|---------------|-------------|--------|--------|-----------|

**Clock**: {Goal} — {4/6/8 segments} — {what fills it}

**PC Leverage**:
- **Offer**: {resource, proof, service, alliance, protection, access}
- **Pressure**: {threat, exposure, sabotage, rival support, legal/social/magical force}
- **Information**: {secret, uncertainty, dependency, or question the PCs can exploit}

**Concessions**:
- {Concession}: requires {condition}; costs {price/risk/compromise}

**Clock Interventions**:
- **Fill**: {PC action that advances the faction clock}
- **Stall/Empty**: {PC action that slows or reverses the clock}
- **Redirect**: {PC action that changes the clock target or creates faction conflict}

## Internal Divisions (if they serve this situation)
## Faction Relationship Map
## PC Goal Intersections
| PC Goal | Faction | Overlap | Tension |
|---------|---------|---------|---------|

## PC Goal Paths
| PC Goal | Viable Approaches | Required Leverage | Key Factions/NPCs | Obstacles | Costs/Risks | Progress Signals | Clock Effects |
|---------|-------------------|-------------------|-------------------|-----------|-------------|------------------|---------------|

## PC-Facing Opportunities
| Opportunity | Why It Matters | Useful Leverage | Factions/NPCs | Risks | Progress Signals |
|-------------|----------------|-----------------|---------------|-------|------------------|

## Default Timeline
## Lore Check
## Open Threads

## Brief

> {brief, verbatim, each line prefixed with `> `}
```

Omit `## Brief` if the brief is empty. Krust appends `## Rounds of Feedback` at runtime — do not author it manually. Section ordering at the footer is: `## Open Threads` → `## Brief` → `## Rounds of Feedback` (krust-managed).

---

## Validation Checklist

- [ ] Central conflict is specific, not generic
- [ ] Every faction has goals at all three timescales
- [ ] Every NPC has a personal goal distinct from their faction's
- [ ] Every key NPC has something they offer or threaten; otherwise they were removed or redesigned
- [ ] At least one PC goal intersects with each faction (if PC goals were provided)
- [ ] If PC goals were provided, every PC goal has at least one concrete path in `## PC Goal Paths`
- [ ] Each PC goal path names leverage, obstacles, costs/risks, progress signals, and clock effects
- [ ] If PC goals were not provided, `## PC-Facing Opportunities` gives playable openings implied by the premise
- [ ] Every faction has PC Leverage, Concessions, and Clock Interventions
- [ ] PC achievement guidance describes tools and pressure points, not scripted solutions or PC decision trees
- [ ] Default timeline shows what each faction does without PC intervention
- [ ] No faction is pure set dressing — each creates pressure on the PCs
- [ ] The "good" side is on the back foot — there's a reason PCs matter
- [ ] The "ugly" faction is a genuine tipping point, not just neutral filler
- [ ] Villain goals are designed through the lens of PC goals — overlap creates conflict
- [ ] Every section present is substantive (empty sections removed, not filled with boilerplate)

---

## Constraints

- **Never invent campaign lore.** If specific lore is needed and not provided, use `[NEEDS GM INPUT]` placeholders.
- **Output sections are conditional.** Only include sections that are substantive — remove empty or boilerplate sections.
- **Nodes are situations, not scenes.** Design the landscape of forces, not a sequence of events.
- **The Framework section governs all creative decisions.** Apply the Three Faction Rule and Proactive Roleplaying framework precisely as documented above.
