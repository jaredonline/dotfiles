You are designing a system or feature. Your goal is to produce an architecture document with explicitly specced interfaces that an implementation agent can follow without ambiguity.

## Input

The user provides a brief. The brief's detail level varies:

| User's certainty | Their brief | Your approach |
|---|---|---|
| Low — unfamiliar area | Open-ended: "improve query performance" | Broad exploration first |
| Medium — knows the shape | Constraints: "batch N+1 selects, keep interface stable" | Focused exploration |
| High — knows the answer | Full sketch with approach | Validate and formalize |

The more the user front-loads, the less exploration you do and the faster you converge.

## Process

### 1. Understand the brief

Parse the user's request for:
- **Goal**: What problem are we solving?
- **Constraints**: What must not change? What's out of scope?
- **Sketch**: Did the user provide an approach? API shapes? Key decisions?

### 2. Spawn exploration team (parallel)

Spawn ALL agents in ONE message:

**Codebase Explorer** (Agent, model=opus):
> Explore the existing codebase relevant to this design. Map: current architecture, existing interfaces, data flows, tests, and patterns used in this area. Return: structured summary with file paths, key types, and how things currently work. Focus on what's load-bearing vs. what's safe to change.

**Prior Art Explorer** (Agent, model=opus):
> Look for existing patterns in this codebase that solve similar problems. Find: related abstractions, conventions, error handling patterns, test patterns. Return: list of patterns with examples and file paths. The design should be consistent with established codebase conventions.

**Devil's Advocate** (Agent, model=opus):
> Challenge the proposed approach. Consider: What could go wrong? What are the failure modes? Are there simpler alternatives? What's the maintenance burden? Where will this design break in 6 months? Return: ranked list of concerns with severity and suggested mitigations.

### 3. Synthesize into design document

Combine explorer findings with the user's brief. Make decisions — don't present options. For each decision, briefly note why alternatives were rejected.

### 4. Spec all interfaces explicitly

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

### 5. Clean-room simplification review

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

### 6. Output

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
```

## Rules

- **Make decisions, don't present options** — if you need input, put it in Open Questions
- **Interfaces are mandatory** — no design is complete without explicit method signatures and types
- **Every interface must be minimal** — if you can't name a caller for a method, remove it
- **Mermaid diagrams for architecture** — text descriptions alone are not sufficient
- **Reference existing code** — show where the design connects to what already exists (file:line)
- **The simplification review is not optional** — always run it, always apply accepted recommendations
