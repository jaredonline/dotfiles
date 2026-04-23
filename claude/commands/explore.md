You are exploring and documenting a system or area of the codebase. Your goal is to produce a structured document that gives a reader (human or agent) a complete mental model.

## Input

The user provides a system, feature, or area to explore. Examples:
- "the auth token rotation system"
- "how the API gateway routes requests"
- "the billing pipeline"

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Create Beads task | No — main agent | Track work before starting |
| 2. Scope exploration | No — main agent | Needs user input to define focus |
| 3. Topic explorers (Data Flow, Schema, Integration, Invariant) | Yes — 4 agents | Independent research areas |
| 4. Synthesize | No — main agent | Combines all explorer output |
| 5. Output | No — main agent | Formats final document |
| 6. Self-validation | No — main agent | Checks output completeness |

## Process

### 1. Create Beads task

Run:

```bash
bd_id=$(krust bd-start task "Explore: [topic]")
```

`krust bd-start` auto-detects mode: under krust (KRUST_BEADS_ID set) it prints the existing task ID, standalone it creates and claims a new task with project-resolved labels. Capture the output into `$bd_id` for subsequent `bd update --notes` calls.

If running under krust, the task inputs (topic, etc.) live in bd metadata — read via `bd show $bd_id --json` and extract from `.metadata.krust.brief`. Standalone, `[topic]` comes from the user's `$ARGUMENTS`.

Skills do not run git directly — krust owns all git operations.

You will reference `$bd_id` in the ## Tracking section of your final output.

### 2. Scope the exploration

Identify the key questions to answer:
- What does this system do?
- What are the request/data flows?
- What are the key invariants and constraints?
- What data stores and schemas are involved?
- What are the integration points with other systems?
- What are the failure modes?

### 3. Spawn topic explorers (parallel)

Spawn ALL explorers in ONE message. Each gets a focused area:

**Data Flow Explorer** (Agent, model=opus):
> Trace the primary data flows through this system. Follow requests end-to-end. Identify entry points, transformations, and where data is stored or forwarded. Return: numbered list of flows with file paths and function names.

**Schema & State Explorer** (Agent, model=opus):
> Find all data stores, schemas, types, and state this system manages. Include database tables, proto definitions, config files, caches. Return: list of schemas with field descriptions and where they're used.

**Integration Explorer** (Agent, model=opus):
> Map all integration points — other services this system calls, services that call it, shared queues, events published/consumed. Return: list of dependencies with direction (inbound/outbound) and protocol.

**Invariant Explorer** (Agent, model=opus):
> Identify constraints, invariants, and implicit rules. Look for: validation logic, error handling patterns, retry policies, ordering guarantees, consistency requirements. Return: list of invariants with the code that enforces them.

### 4. Synthesize

Combine all explorer findings into a single structured document. Resolve conflicts between explorers. Fill gaps by reading additional files if needed.

### 5. Output

Write the exploration document to a working file. Under krust the wrapper pre-computed `$KRUST_OUT` as a starting path; you may write to `$KRUST_OUT` directly OR to a temp path of your choice — the harness handles relocation via the slug you declare in the next step. Standalone, write anywhere (e.g. `/tmp/exploration-draft.md`).

Then emit the artifact:

```bash
krust artifact explorations <slug> <path-you-wrote-to>
```

`<slug>` is a kebab-case version of the system/area name (max 50 chars). Under krust, `krust artifact` emits an action JSON that the wrapper consumes to rename/commit the file; standalone, it writes the file to `$COCKPIT_DIR/state/explorations/<slug>.md` and commits+pushes directly. Either way — same prose.

Before writing the ## Tracking section, close out the beads task:

```bash
krust bd-finish "$bd_id"
```

`krust bd-finish` is a no-op under krust (the wrapper closes the task on approval) and closes the bd task directly when standalone.

Produce a markdown document with these sections:

```markdown
# [System Name]

## Overview
One paragraph: what it does, why it exists.

## Architecture
Mermaid diagram showing key components and their relationships.

## Request/Data Flows
Numbered flows, each with:
- Entry point (file:line)
- Steps with file references
- Terminal state

## Data Stores & Schemas
Tables, caches, queues — with key fields and purpose.

## Integration Points
What this system talks to and what talks to it.

## Key Invariants
Constraints the system maintains, with the code that enforces them.

## Failure Modes
What breaks and how the system handles it.

## Tracking
- Beads: <task-id> — closed
```

### 6. Self-validation

Before presenting the final document, verify:

- [ ] Every section in the output template has content (or an explicit "N/A — [reason]")
- [ ] Every claim references a specific file and function/line
- [ ] No inference is presented as established fact — column names, variable names, and patterns are labeled as suggestive, not definitive
- [ ] Architecture diagram exists and matches the described components
- [ ] Data flows are end-to-end (entry point → terminal state), not fragments
- [ ] Integration points list direction (inbound/outbound) and protocol
- [ ] No section is a restatement of another — each adds distinct information
- [ ] ## Tracking section includes Beads task ID
- [ ] `krust artifact explorations <slug> <path>` was called with the final document path

If any check fails, go back and fill the gap before presenting.

## Rules

- Every claim must reference a specific file and function/line
- Use mermaid diagrams for architecture and complex flows
- Keep it factual — document what IS, not what should be
- If an area is unclear or undocumented, say so explicitly
- **Distinguish evidence from inference.** A claim backed by code you read is evidence. A claim derived from a column name, variable name, or pattern match is inference. Label inferences explicitly: "the column name suggests…" not "this is…". When the user or a coworker will act on your output (PR comments, Slack replies, Asana updates), only state what you can cite — bad inferences erode trust faster than gaps do.
- **When answering coworker questions or drafting external communication**, every factual claim must have a source you can point to (file:line, URL, error message). If you can't find one, say "I couldn't verify this" rather than presenting inference as fact.
