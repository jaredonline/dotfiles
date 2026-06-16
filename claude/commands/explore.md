You are exploring and documenting a system or area of the codebase. Your goal is to produce a structured document that gives a reader (human or agent) a complete mental model.

## Input

The topic to explore comes from `$ARGUMENTS` — a system, feature, or area. Examples:
- "the auth token rotation system"
- "how the API gateway routes requests"
- "the billing pipeline"

### Optional: --redesign <path>

If the invocation includes `--redesign <path>`, read the prior exploration at
`<path>` before spawning explorers. Use it to steer the new run from a different
angle — identify what the prior doc covered well, what it missed or got wrong,
and bias the four topic explorers (Data Flow, Schema, Integration, Invariant)
toward the gaps. Do not just repeat the prior doc.

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Scope exploration | No — main agent | Needs the topic to define focus |
| 2. Topic explorers (Data Flow, Schema, Integration, Invariant, Devil's Advocate) | Yes — 5 agents | Independent research areas + skeptic |
| 3. Synthesize | No — main agent | Combines all explorer output |
| 4. Output | No — main agent | Writes the doc to the cockpit repo and pushes |
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

Spawn ALL 5 explorers in ONE assistant message using the `Agent` tool. Each is a synchronous, blocking call — multiple `Agent` tool uses in a single message run concurrently and the harness blocks the turn until every `tool_result` returns. **Do not set `run_in_background: true`. Do not use `TeamCreate` or any team-lifecycle tools** — async/teams semantics cause sub-agent completions to arrive as `task_notification` events that the lead can narrate and end its turn on without writing the artifact. Each gets a focused area:

**Data Flow Explorer** (Agent, model=opus):
> Trace the primary data flows through this system. Follow requests end-to-end. Identify entry points, transformations, and where data is stored or forwarded. Return: numbered list of flows with file paths and function names.

**Schema & State Explorer** (Agent, model=opus):
> Find all data stores, schemas, types, and state this system manages. Include database tables, proto definitions, config files, caches. Return: list of schemas with field descriptions and where they're used.

**Integration Explorer** (Agent, model=opus):
> Map all integration points — other services this system calls, services that call it, shared queues, events published/consumed. Return: list of dependencies with direction (inbound/outbound) and protocol.

**Invariant Explorer** (Agent, model=opus):
> Identify constraints, invariants, and implicit rules. Look for: validation logic, error handling patterns, retry policies, ordering guarantees, consistency requirements. Return: list of invariants with the code that enforces them.

**Devil's Advocate** (Agent, model=opus):
> Challenge the other explorers' findings. Look for: undocumented behavior, race conditions, edge cases not covered by tests, claims that look like inference dressed up as fact, integration assumptions that may not hold under load. Where possible, write small probe scripts or run targeted tests to falsify a claim rather than accept it. Return: ranked list of concerns with the specific finding being challenged, the falsification attempt, and the result.

#### Execution verification protocol (applies to ALL five explorers)

Each explorer's prompt implicitly carries this protocol. When a claim can be verified by running code, prefer execution over inference:

- **Targeted tests**: `go test ./path/to/package -run TestRelevant -v -count=1 -short -timeout 30s` (always scope to a package + test name; never `go test ./...`)
- **Probe scripts**: small ephemeral scripts to confirm a behavior. Run, capture the result in your report, then delete.
- **Benchmarks** for performance claims: `go test -bench=BenchmarkX -benchmem`
- **External evidence**: if MCP tools are available (Datadog, Slack), query for production data or discussion context.

Each finding an explorer reports must carry one of four labels:
- `code reading` — derived only from reading source files
- `execution verified` — confirmed by running code (test, probe, benchmark)
- `production data` — confirmed by Datadog metrics or other production signals
- `discussion context` — supported by Slack threads, design docs, or other discussion

Discover available MCP tools once at the start of step 2 with a single call: `ToolSearch(query="datadog OR slack OR statsig OR mcp", max_results=20)`. Extract just the tool names into a comma-separated list and inject into each teammate prompt under a `## Available External Tools` heading (e.g., `## Available External Tools\nmcp__datadog__query_metrics, mcp__slack__slack_search_public, ...` — or the literal string `none` if nothing matched). If a tool isn't available, the teammate notes the gap rather than working around it.

### 3. Synthesize

Combine all explorer findings into a single structured document. Resolve conflicts between explorers — when two findings conflict, prefer the one with stronger evidence using this precedence: `execution verified` and `production data` outrank `code reading` and `discussion context`; between two findings of the same tier, use judgment (cite both in the Evidence Summary). Fill gaps by reading additional files if needed; if a high-value claim is `code reading` only and trivial to verify, run the verification yourself before recording it.

### 4. Output

Write the exploration document to the cockpit repo and push it.

```bash
COCKPIT_DIR="${COCKPIT_DIR:-$HOME/ai-cockpit}"
slug="<kebab-case system/area name, max 50 chars>"

# Don't clobber an existing exploration — bump to -v2, -v3, … if the slug is taken.
out="$COCKPIT_DIR/state/explorations/$slug.md"
if [ -e "$out" ]; then
  n=2
  while [ -e "$COCKPIT_DIR/state/explorations/$slug-v$n.md" ]; do n=$((n+1)); done
  slug="$slug-v$n"
  out="$COCKPIT_DIR/state/explorations/$slug.md"
fi
```

Write the document (below) to `"$out"` with the `Write` tool, then commit and push:

```bash
git -C "$COCKPIT_DIR" add "state/explorations/$slug.md"
git -C "$COCKPIT_DIR" commit -m "explore: $slug"
git -C "$COCKPIT_DIR" push
```

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

## Evidence Summary

| Finding | Verification | Confidence |
|---------|-------------|------------|
| <claim> | <how it was checked, e.g., `go test ./pkg -run TestX` passed> | execution verified |
| <claim> | Read <file>:<line> | code reading |
| <claim> | Datadog dashboard <name> | production data |
| <claim> | Slack thread <link or summary> | discussion context |

Use one of the four labels: `code reading`, `execution verified`, `production data`, `discussion context`.

## Investigation Log

The operational record of what was run during this exploration. Distinct from the Evidence Summary above: the Summary cites the verification *behind each finding*, while this Log captures investigations that span multiple findings, that produced negative or null results, or that informed the exploration without surfacing as a discrete claim. If every entry here also appears in the Evidence Summary, this section may be omitted with a note: "All investigations are reflected in the Evidence Summary."

- **Tests run**: command + pass/fail + brief result
- **Probe scripts**: what behavior was checked, the result (script itself was deleted after running)
- **Benchmarks**: command + numeric result
- **External data**: Datadog queries run, Slack threads consulted

## Brief

> <brief, verbatim, each line prefixed with `> `>
```

If the brief is empty, omit the `## Brief` section entirely.

### 5. Self-validation

Before presenting the final document, verify:

- [ ] Every section in the output template has content (or an explicit "N/A — [reason]")
- [ ] Every claim references a specific file and function/line
- [ ] No inference is presented as established fact — column names, variable names, and patterns are labeled as suggestive, not definitive
- [ ] Architecture diagram exists and matches the described components
- [ ] Data flows are end-to-end (entry point → terminal state), not fragments
- [ ] Integration points list direction (inbound/outbound) and protocol
- [ ] No section is a restatement of another — each adds distinct information
- [ ] The document was written to `$COCKPIT_DIR/state/explorations/<slug>.md` and committed + pushed
- [ ] Five explorers were spawned (Data Flow, Schema, Integration, Invariant, Devil's Advocate)
- [ ] Every claim in the Evidence Summary carries one of the four labels
- [ ] Investigation Log lists every test/probe/benchmark/external query that was actually run
- [ ] No probe scripts left on disk

If any check fails, go back and fill the gap before presenting.

## Rules

- Every claim must reference a specific file and function/line
- Use mermaid diagrams for architecture and complex flows
- Keep it factual — document what IS, not what should be
- If an area is unclear or undocumented, say so explicitly
- **Distinguish evidence from inference.** A claim backed by code you read is evidence. A claim derived from a column name, variable name, or pattern match is inference. Label inferences explicitly: "the column name suggests…" not "this is…". When the user or a coworker will act on your output (PR comments, Slack replies, Asana updates), only state what you can cite — bad inferences erode trust faster than gaps do.
- **When answering coworker questions or drafting external communication**, every factual claim must have a source you can point to (file:line, URL, error message). If you can't find one, say "I couldn't verify this" rather than presenting inference as fact.
