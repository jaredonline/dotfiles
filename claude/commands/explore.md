You are exploring and documenting a system or area of the codebase. Your goal is to produce a structured document that gives a reader (human or agent) a complete mental model.

## Input

The user provides a system, feature, or area to explore. Examples:
- "the auth token rotation system"
- "how the API gateway routes requests"
- "the billing pipeline"

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Scope exploration | No — main agent | Needs user input to define focus |
| 2. Topic explorers (Data Flow, Schema, Integration, Invariant) | Yes — 4 agents | Independent research areas |
| 3. Synthesize | No — main agent | Combines all explorer output |
| 4. Output | No — main agent | Formats final document |
| 5. Self-validation | No — main agent | Checks output completeness |

## Process

### 1. Scope the exploration

Identify the key questions to answer:
- What does this system do?
- What are the request/data flows?
- What are the key invariants and constraints?
- What data stores and schemas are involved?
- What are the integration points with other systems?
- What are the failure modes?

### 2. Spawn topic explorers (parallel)

Spawn ALL explorers in ONE message. Each gets a focused area:

**Data Flow Explorer** (Agent, model=opus):
> Trace the primary data flows through this system. Follow requests end-to-end. Identify entry points, transformations, and where data is stored or forwarded. Return: numbered list of flows with file paths and function names.

**Schema & State Explorer** (Agent, model=opus):
> Find all data stores, schemas, types, and state this system manages. Include database tables, proto definitions, config files, caches. Return: list of schemas with field descriptions and where they're used.

**Integration Explorer** (Agent, model=opus):
> Map all integration points — other services this system calls, services that call it, shared queues, events published/consumed. Return: list of dependencies with direction (inbound/outbound) and protocol.

**Invariant Explorer** (Agent, model=opus):
> Identify constraints, invariants, and implicit rules. Look for: validation logic, error handling patterns, retry policies, ordering guarantees, consistency requirements. Return: list of invariants with the code that enforces them.

### 3. Synthesize

Combine all explorer findings into a single structured document. Resolve conflicts between explorers. Fill gaps by reading additional files if needed.

### 4. Output

Write the exploration document to the cockpit state directory:
- If `$COCKPIT_DIR` is set: `mkdir -p "$COCKPIT_DIR/state/explorations"` and write the document to `$COCKPIT_DIR/state/explorations/<slug>.md` where `<slug>` is a kebab-case version of the system/area name (max 50 chars).
- If `$COCKPIT_DIR` is not set: warn "COCKPIT_DIR not set — exploration written to conversation only"

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
```

### 5. Self-validation

Before presenting the final document, verify:

- [ ] Every section in the output template has content (or an explicit "N/A — [reason]")
- [ ] Every claim references a specific file and function/line
- [ ] Architecture diagram exists and matches the described components
- [ ] Data flows are end-to-end (entry point → terminal state), not fragments
- [ ] Integration points list direction (inbound/outbound) and protocol
- [ ] No section is a restatement of another — each adds distinct information

If any check fails, go back and fill the gap before presenting.

## Rules

- Every claim must reference a specific file and function/line
- Use mermaid diagrams for architecture and complex flows
- Keep it factual — document what IS, not what should be
- If an area is unclear or undocumented, say so explicitly
