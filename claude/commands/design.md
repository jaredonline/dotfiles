You are designing a system or feature. Your goal is to produce an architecture document with explicitly specced interfaces that an implementation agent can follow without ambiguity.

## Input

The user provides a brief. The brief's detail level varies:

| User's certainty | Their brief | Your approach |
|---|---|---|
| Low — unfamiliar area | Open-ended: "improve query performance" | Broad exploration first |
| Medium — knows the shape | Constraints: "batch N+1 selects, keep interface stable" | Focused exploration |
| High — knows the answer | Full sketch with approach | Validate and formalize |

- **--redesign <path>** (optional) — path to a prior design doc for revision. When provided, the brief describes what to CHANGE.

The more the user front-loads, the less exploration you do and the faster you converge.

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Create Beads task | No — main agent | Track work before starting |
| 2. Understand brief | No — main agent | Needs user input |
| 3. Parallel exploration via Agent ×3 | Yes — 3 agents in one message | Independent research, fire-and-forget |
| 4. Synthesize | No — main agent | Combines findings into design |
| 5. Spec interfaces | No — main agent | Depends on synthesis |
| 6. Simplification review | No — 1 agent (clean-room) | Isolated from exploration context |
| 7. Output | No — main agent | Writes the document |
| 8. Commit the artifact (REQUIRED) | No — main agent | Persists the design and closes the task |
| 9. Self-validation | No — main agent | Final audit |

## Process

### 1. Create Beads task

Create and claim a task to track the work:

```bash
bd_id=$(bd create --title="Design: [topic]" --type=task --priority=2 | grep -oE '[a-z]+-[0-9]+')
bd update "$bd_id" --claim
```

`[topic]` comes from the user's `$ARGUMENTS`. Capture the task ID into `$bd_id` for subsequent `bd update --notes` calls. The brief is `$ARGUMENTS`.

You will reference `$bd_id` in the ## Tracking section of your final output.

### 2. Understand the brief

Parse the user's request for:
- **Goal**: What problem are we solving?
- **Constraints**: What must not change? What's out of scope?
- **Sketch**: Did the user provide an approach? API shapes? Key decisions?

If `--redesign <path>` is provided, read the file at that path and use it as the starting point. Parse the prior doc's architecture, interfaces, decisions, and invariants as the baseline. The brief describes changes to make, not the full design.

Before moving to exploration, check the project root for foundational documents that define how this project thinks about design:

- `ARCHITECTURE.md` — system structure, component relationships, boundaries
- `PRINCIPLES.md` — design values, conventions, non-negotiable patterns
- `PHILOSOPHY.md` — broader design stance, trade-off preferences

Read each file that exists. Skip any that don't — not every project has all three. Do not re-read CLAUDE.md (it's auto-loaded by the harness).

Use what you learn to:
1. Refine your understanding of the brief — does the project's philosophy constrain or expand the solution space?
2. Write more targeted explorer prompts — tell explorers what principles to validate against.
3. Inform synthesis — designs should align with stated principles unless the brief explicitly aims to change them.

### 3. Spawn exploration agents (parallel)

Spawn ALL 3 explorers in ONE assistant message using the `Agent` tool. Each is a synchronous, blocking call — multiple `Agent` tool uses in a single message run concurrently and the harness blocks the turn until every `tool_result` returns. **Do not set `run_in_background: true`. Do not use `TeamCreate` or any team-lifecycle tools** — async/teams semantics cause sub-agent completions to arrive as `task_notification` events that the lead can narrate and end its turn on without writing the artifact.

**Codebase Explorer:**
```
Agent(
  description="Codebase exploration",
  subagent_type="Explore",
  prompt="Explore the existing codebase relevant to this design. Map: current architecture, existing interfaces, data flows, tests, and patterns used in this area. Return: structured summary with file paths, key types, and how things currently work. Focus on what's load-bearing vs. what's safe to change."
)
```

**Prior Art Explorer:**
```
Agent(
  description="Prior art exploration",
  subagent_type="Explore",
  prompt="Look for existing patterns in this codebase that solve similar problems. Find: related abstractions, conventions, error handling patterns, test patterns. Return: list of patterns with examples and file paths. The design should be consistent with established codebase conventions."
)
```

**Devil's Advocate:**
```
Agent(
  description="Devil's advocate critique",
  prompt="Challenge the proposed approach. Consider: What could go wrong? What are the failure modes? Are there simpler alternatives? What's the maintenance burden? Where will this design break in 6 months? Return: ranked list of concerns with severity and suggested mitigations."
)
```

Do NOT advance to step 4 until every agent has returned its result. The harness blocks the assistant turn on outstanding tool calls — you'll receive all three results before continuing.

### 4. Synthesize into design document

Combine explorer findings with the user's brief. Weight findings flagged by 2+ explorers highest (consensus) and surface them first in the relevant Key Decisions row; integrate single-explorer findings after.

Make decisions — don't present options. For each decision, briefly note why alternatives were rejected.

As architect, also answer this decomposition checkpoint explicitly: **could this design ship in N ordered PRs?** If yes, identify the minimum coherent first PR and what each subsequent PR adds. The answer drives whether you emit `## Decomposition Hints` in step 7 — if the architecture naturally decomposes into >= 2 ordered chunks, populate the section; otherwise omit it. Forcing a decomposition that introduces concepts outside the Architecture section is a smell — when in doubt, omit.

### 5. Spec all interfaces explicitly

This is the most important step. For every boundary in the design:

**External APIs** — HTTP endpoints, gRPC services, CLI commands:
```
POST /api/v1/resource
  Request: { field: type, ... }
  Response: { field: type, ... }
  Errors: 400 (why), 404 (why), 500 (why)
  Auth: required/optional, mechanism
```

**Internal interfaces** — language-level types and method signatures:
```
type ServiceName interface {
    MethodName(ctx context.Context, req RequestType) (*ResponseType, error)
}
```

**Data schemas** — new or modified tables, protos, configs:
```
table: name
  column: type — purpose
```

Keep interfaces minimal. If a method isn't needed by the design, don't add it.

### 6. Clean-room simplification review

After the design is written, spawn a simplification reviewer with ONLY the design document — no access to the exploration context:

**Simplifier** (Agent, model=opus):
> You are reviewing this design document for unnecessary complexity. You have no context beyond this document. For each element, ask:
> - Can this interface be removed without losing functionality?
> - Are there unnecessary layers of abstraction?
> - Could two components be merged?
> - Is any flexibility speculative (no concrete use case)?
>
> Return a structured simplification report in this exact format:
>
> ```markdown
> # Simplification Report
>
> ## Summary
> - Recommendations: <count>
> - By type: <merge:N | extract:N | remove:N | restructure:N | clarify:N>
>
> ## Recommendations
>
> ### 1. [Short title]
> **Type**: Merge | Extract | Remove | Restructure | Clarify
> **Location**: [Section/component in design doc]
> **Current State**: [Quote or describe]
> **Proposed Change**: [Specific simplification]
> **Rationale**: [Why this improves the design]
> **Risk Assessment**:
> - Functionality impact: None | Low | Medium | High
> - Correctness impact: None | Low | Medium | High
>
> ## Recommendations NOT Made
> - **[Area]**: [Why simplification would be harmful]
> ```

For each recommendation: ACCEPT, REJECT (with reason), or MODIFY. Apply accepted simplifications to the design.

### 7. Output

Pick a `$slug` — a kebab-case version of the feature/system name (max 50 chars) — and write the design document to `$COCKPIT_DIR/state/designs/design-$slug.md` (run `mkdir -p "$COCKPIT_DIR/state/designs"` first if needed). Reuse this same `$slug` in step 8.

Produce a markdown document with these sections:

```markdown
# Design: [Feature/System Name]

## Problem
What we're solving and why.

## Constraints
What must not change. What's out of scope.

## Architecture
Mermaid diagram of components and their relationships.

## Interfaces

### External APIs
[Full specs with request/response/errors]

### Internal Interfaces
[Language-level types and method signatures]

### Data Schemas
[New or modified schemas]

## Data Flow
Step-by-step flow for each key operation, referencing interfaces above.

## Decomposition Hints

The architecture above naturally decomposes into these chunks for stacked delivery:

1. **<layer-name>** — <one sentence>
   - **Merge-alone invariant**: <what must be true for this layer to safely ship alone>
   - **Touches**: <files/components>

2. **<layer-name>** — ...

Emit this section ONLY when the design decomposes into >= 2 ordered chunks (per the step-4 checkpoint). Otherwise omit it entirely. Each hint MUST have a `Merge-alone invariant` (a property that holds if this layer ships alone) and a `Touches` line listing files or components. Number hints starting at 1 in delivery order. Every hint must reference at least one component from the `## Architecture` section — hints that introduce new concepts are forced decomposition; remove them and, if none remain, omit the section.

## Key Decisions
| Decision | Chosen | Rejected | Why |
|----------|--------|----------|-----|

## Invariants
Constraints the implementation must maintain.

## Open Questions
Anything that needs human input before implementation.

## Tracking
- Beads: <task-id> — closed

## Next Step
Run /plan to create the task graph for implementation.

## Brief

> <brief, verbatim, each line prefixed with `> `>
```

Omit the `## Brief` section entirely if the brief is empty. Preserve the brief verbatim, including newlines — each line gets a `> ` prefix to form a markdown blockquote.

### 8. Commit the artifact (REQUIRED)

After writing the document, persist it and close the task. Do not exit the turn before these succeed.

```bash
# 1. Commit and push the design document.
git add "$COCKPIT_DIR/state/designs/design-$slug.md"
git -C "$COCKPIT_DIR" commit -m "Add design: $slug"
git -C "$COCKPIT_DIR" push

# 2. If this design was informed by an exploration in $COCKPIT_DIR/state/explorations/,
#    move it into the finished/ subdirectory to keep the active set clean.
if [ -n "$exploration_file" ]; then
  mkdir -p "$COCKPIT_DIR/state/explorations/finished"
  git -C "$COCKPIT_DIR" mv "$exploration_file" "$COCKPIT_DIR/state/explorations/finished/$(basename "$exploration_file")"
  git -C "$COCKPIT_DIR" commit -m "Archive exploration: finished $slug"
  git -C "$COCKPIT_DIR" push
fi

# 3. Close the Beads task.
bd close "$bd_id"
```

### 9. Self-validation

Before presenting the final document, verify:

- [ ] The design was written to `$COCKPIT_DIR/state/designs/design-$slug.md`, committed, and pushed
- [ ] `bd close "$bd_id"` was called
- [ ] Every section in the output template has content (or an explicit "N/A — [reason]")
- [ ] All interfaces have full signatures with types, parameters, and error cases
- [ ] Key Decisions table has at least one rejected alternative per decision
- [ ] Architecture diagram exists and matches the described components
- [ ] Invariants section lists concrete constraints, not vague goals
- [ ] If `## Decomposition Hints` is present: >= 2 hints, each with `Merge-alone invariant` and `Touches`, numbered from 1 in delivery order, and every hint references a component from `## Architecture`. If the design doesn't decompose, the section is omitted entirely
- [ ] Open Questions are genuine blockers, not deferred decisions you could have made
- [ ] ## Tracking section includes Beads task ID
- [ ] Step 3 spawned all 3 explorers in a single assistant message via `Agent` (true parallelism, no team lifecycle)
- [ ] Synthesis (step 4) ran consensus detection and surfaced 2+-explorer findings first
- [ ] Simplifier output (step 6) uses Type / Location / Current State / Proposed Change / Rationale / Risk Assessment format

If any check fails, fix it before presenting.

## Rules

- **Make decisions, don't present options** — if you need input, put it in Open Questions
- **Interfaces are mandatory** — no design is complete without explicit method signatures and types
- **Every interface must be minimal** — if you can't name a caller for a method, remove it
- **Mermaid diagrams for architecture** — text descriptions alone are not sufficient
- **Reference existing code** — show where the design connects to what already exists (file:line)
- **The simplification review is not optional** — always run it, always apply accepted recommendations
