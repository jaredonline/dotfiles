You are designing a system or feature. Your goal is to produce an architecture document with explicitly specced interfaces that an implementation agent can follow without ambiguity.

## Input

The user provides a brief. The brief's detail level varies:

| User's certainty | Their brief | Your approach |
|---|---|---|
| Low — unfamiliar area | Open-ended: "improve query performance" | Broad exploration first |
| Medium — knows the shape | Constraints: "batch N+1 selects, keep interface stable" | Focused exploration |
| High — knows the answer | Full sketch with approach | Validate and formalize |

The more the user front-loads, the less exploration you do and the faster you converge.

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Create Beads task | No — main agent | Track work before starting |
| 2. Understand brief | No — main agent | Needs user input |
| 3. Exploration team | Yes — 3 agents | Independent research |
| 4. Synthesize | No — main agent | Combines findings into design |
| 5. Spec interfaces | No — main agent | Depends on synthesis |
| 6. Simplification review | No — 1 agent (clean-room) | Isolated from exploration context |
| 7. Output | No — main agent | Formats final document |
| 8. Self-validation | No — main agent | Checks output completeness |

## Process

### 1. Create Beads task

**When running under krust** (`$KRUST_BEADS_ID` is set):
- Read inputs from beads metadata: `bd show $KRUST_BEADS_ID --json`
- Extract `artifact_path`, `brief` from `.metadata.krust`
- Do NOT create a new beads task — the wrapper already created one
- Do NOT run any git operations anywhere in this skill
- Skip the "Project labeling" block entirely

**When running standalone** (`$KRUST_BEADS_ID` is not set):

**Project labeling**: Read `$COCKPIT_DIR/project-tree.json` (skip if missing or `COCKPIT_DIR` unset). Review the project list to understand the landscape of active projects and their labels. Determine which project this task belongs to by matching `cwd` against project `path` fields and matching the task topic against project names. If exactly one project matches, use its `labels` array. If ambiguous or no match, ask the user which project this is for. Store the resolved labels for all `bd create` calls in this skill invocation.

Run `bd create --title="Design: [topic]" --description="[brief description of what is being designed]" --type=task --labels=<resolved-labels>` and store the returned task ID.
Claim it: `bd update <id> --claim`.
You will reference the task ID in the ## Tracking section of your final output.

### 2. Understand the brief

Parse the user's request for:
- **Goal**: What problem are we solving?
- **Constraints**: What must not change? What's out of scope?
- **Sketch**: Did the user provide an approach? API shapes? Key decisions?

### 3. Spawn exploration team (parallel)

Spawn ALL agents in ONE message:

**Codebase Explorer** (Agent, model=opus):
> Explore the existing codebase relevant to this design. Map: current architecture, existing interfaces, data flows, tests, and patterns used in this area. Return: structured summary with file paths, key types, and how things currently work. Focus on what's load-bearing vs. what's safe to change.

**Prior Art Explorer** (Agent, model=opus):
> Look for existing patterns in this codebase that solve similar problems. Find: related abstractions, conventions, error handling patterns, test patterns. Return: list of patterns with examples and file paths. The design should be consistent with established codebase conventions.

**Devil's Advocate** (Agent, model=opus):
> Challenge the proposed approach. Consider: What could go wrong? What are the failure modes? Are there simpler alternatives? What's the maintenance burden? Where will this design break in 6 months? Return: ranked list of concerns with severity and suggested mitigations.

### 4. Synthesize into design document

Combine explorer findings with the user's brief. Make decisions — don't present options. For each decision, briefly note why alternatives were rejected.

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
> For each recommendation, explain what functionality/correctness/invariant would be affected.
> Return: numbered list of simplification recommendations with impact analysis.

For each simplification recommendation: ACCEPT, REJECT (with reason), or MODIFY. Apply accepted simplifications to the design.

### 7. Output

**When running under krust** (`$KRUST_BEADS_ID` is set):
1. Do NOT run `bd close` — the wrapper handles task lifecycle
2. Write the artifact to `artifact_path` from beads metadata
3. `bd update $KRUST_BEADS_ID --set-metadata='artifact_written=true'`
4. If an exploration was consumed, emit action JSON to `$ACTIONS_DIR`:
   ```bash
   echo '{"type":"archive_exploration","file":"<filename>","reason":"finished: <slug>"}' > "$ACTIONS_DIR/archive.json"
   ```
   `bd update $KRUST_BEADS_ID --set-metadata='actions_emitted=true'`
5. `bd update $KRUST_BEADS_ID --set-metadata='skill_complete=true'`
6. Skip git commit/push and archive script steps

**When running standalone** (`$KRUST_BEADS_ID` is not set):

Before writing the Tracking section, run `bd close <task-id>`.

Write the design document to the cockpit state directory:
1. If `$COCKPIT_DIR` is unset or empty, stop: "COCKPIT_DIR is not set. Set it before running /design."
2. `mkdir -p "$COCKPIT_DIR/state/designs"` and write the document to `$COCKPIT_DIR/state/designs/design-<slug>.md` where `<slug>` is a kebab-case version of the feature/system name (max 50 chars).
3. **Commit the new design doc:**
   ```bash
   git -C "$COCKPIT_DIR" add "state/designs/design-<slug>.md"
   git -C "$COCKPIT_DIR" commit -m "design: <slug>"
   git -C "$COCKPIT_DIR" push
   ```
   If any git command fails, warn the user but continue.
4. **Archive consumed exploration.** If this design was informed by an exploration document in `$COCKPIT_DIR/state/explorations/`, archive it:
   ```bash
   ~/.claude/scripts/cockpit-archive.sh explorations <exploration-file>.md "finished: <slug>"
   ```
   If no exploration was consumed, skip this step.

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
```

### 8. Self-validation

Before presenting the final document, verify:

- [ ] Every section in the output template has content (or an explicit "N/A — [reason]")
- [ ] All interfaces have full signatures with types, parameters, and error cases
- [ ] Key Decisions table has at least one rejected alternative per decision
- [ ] Architecture diagram exists and matches the described components
- [ ] Invariants section lists concrete constraints, not vague goals
- [ ] Open Questions are genuine blockers, not deferred decisions you could have made
- [ ] ## Tracking section includes Beads task ID

If any check fails, fix it before presenting.

## Rules

- **Make decisions, don't present options** — if you need input, put it in Open Questions
- **Interfaces are mandatory** — no design is complete without explicit method signatures and types
- **Every interface must be minimal** — if you can't name a caller for a method, remove it
- **Mermaid diagrams for architecture** — text descriptions alone are not sufficient
- **Reference existing code** — show where the design connects to what already exists (file:line)
- **The simplification review is not optional** — always run it, always apply accepted recommendations
