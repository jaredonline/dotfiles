You are writing a design document for human review. Your goal is to produce a clear, thorough document that explains context, problem, constraints, goals, trade-offs, and milestones — suitable for review by engineers, leadership, and cross-functional stakeholders.

## Input

The user provides a topic or brief for the design document. The brief may range from a single sentence to a detailed specification.

If no topic is provided, stop and ask: "What should the design document cover?"

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Create epic | No — main agent | Track work before starting |
| 2. Gather context | No — main agent | Interactive — needs user input |
| 3. Exploration | Yes — 2 agents | Independent research |
| 4. Draft | No — main agent | Synthesizes findings + user input |
| 5. Review panel | Yes — 4 agents | Independent reviewer perspectives |
| 6. Revise | No — main agent | Incorporates review feedback |
| 7. Self-validation | No — main agent | Catches issues before presentation |
| 8. Present for approval | No — main agent | Human review checkpoint |
| 9. Finalize and output | No — main agent | Writes final document |

## Process

### 1. Create epic

Create an epic to track the design doc effort:

bd create --title="Design Doc: [topic]" --description="Human-readable design document for [topic]" --type=epic --priority=2

Claim it: `bd update <epic-id> --claim`.

File child tasks under this epic as work is discovered throughout the process.

### 2. Gather context (interactive)

Ask the user clarifying questions to build understanding. Adapt based on what the brief already covers — skip topics the user has already addressed, add follow-ups when answers reveal complexity.

Topics to cover (ask about gaps, not all of these):
- **Problem**: What problem does this solve? Who experiences it?
- **Current state**: What exists today? Why is it insufficient?
- **Constraints**: Hard limits — timeline, compatibility, regulatory, budget
- **Scope**: What is explicitly out of scope? Anti-goals?
- **Direction**: Preferred approach, or exploring options?
- **Success criteria**: What does success look like? How measured?

Ask in small batches (2-3 questions), building on previous answers. When you have enough context, proceed to exploration.

### 3. Exploration team (parallel)

Spawn ALL agents in ONE message:

**Codebase Explorer** (Agent, model=opus):
> Explore the existing codebase relevant to [topic], including integration points with external systems. Map: current architecture, existing interfaces, data flows, patterns, and dependencies. Identify what's load-bearing vs. safe to change. Return: structured summary with file paths, key types, and how things currently work.

**Prior Art & Alternatives Explorer** (Agent, model=opus):
> Research alternative approaches to [problem]. Look for: existing patterns in this codebase that solve similar problems, industry-standard approaches, and known trade-offs. Return: 3-5 viable approaches with pros/cons for each.

File child tasks for areas needing deeper exploration:
bd create --title="Explore: [area]" --type=task --parent=<epic-id>

Close each when its exploration completes.

### 4. Draft the document

Write the first draft following the output template in step 9.

**Author guidelines:**
- Write for humans — explain context a new team member would need.
- Lead with "why" — problem and motivation before solution.
- Be opinionated — make recommendations, defend them, show what you rejected.
- Use Mermaid diagrams for architecture and data flow.
- Be specific about trade-offs — name what you're giving up.
- Milestones must have concrete deliverables, not vague phases.

### 5. Review panel (parallel)

Spawn ALL reviewers in ONE message. Pass each the full draft.

Each reviewer first assesses whether their perspective is relevant to this design. If not relevant, return: "Not applicable — [reason why this design doesn't touch my domain]."

**Executive Reviewer** (Agent, model=opus):
> You are a VP of Engineering reviewing a design document. Assess relevance first — if this design doesn't warrant executive-level scrutiny (e.g., small internal tooling), say so.
>
> If relevant, you care about: business justification, resource investment, risk to existing systems, timeline realism, organizational alignment. You do NOT care about implementation details.
>
> For each concern: what's wrong, why it matters, what would fix it.
> Return: numbered list of concerns, or "Not applicable — [reason]", or "No concerns — [why it's solid]".

**Staff Engineer Reviewer** (Agent, model=opus):
> You are a Staff Engineer reviewing a design document. You care about: technical correctness, system design quality, whether trade-offs are well-reasoned, whether alternatives were fairly evaluated, maintainability, and unnecessary complexity.
>
> For each concern: what's wrong, why it matters, what would fix it. Challenge assumptions.
> Return: numbered list of concerns, or "No concerns — [why it's solid]".

**Security Engineer Reviewer** (Agent, model=opus):
> You are a Security Engineer reviewing a design document. Assess relevance first — if this design doesn't handle user data, auth, or external input, say so.
>
> If relevant, you care about: threat vectors, data privacy, auth/authz, input validation, secrets management, compliance, abuse potential.
>
> For each concern: the threat, severity (critical/high/medium/low), recommended mitigation.
> Return: numbered list of concerns with severity, or "Not applicable — [reason]", or "No concerns — [why it's safe]".

**SRE Reviewer** (Agent, model=opus):
> You are an SRE reviewing a design document. Assess relevance first — if this design doesn't affect production systems, say so.
>
> If relevant, you care about: operational complexity, monitoring/observability, failure modes, rollback strategy, capacity planning, deployment risk, on-call burden.
>
> For each concern: the operational risk, impact, recommended mitigation.
> Return: numbered list of concerns, or "Not applicable — [reason]", or "No concerns — [why it's low-risk]".

### 6. Revise

For each reviewer's feedback:
- **Accept**: Incorporate into the document. Note in the Revision Log.
- **Reject**: Explain why in the Revision Log.
- **Escalate**: If the concern raises a question you can't answer, add to Open Questions.

File child tasks for work discovered during review:
bd create --title="[discovered work item]" --type=task --parent=<epic-id>

### 7. Self-validation

Before presenting to the user, verify:

- [ ] Context section provides background for a new team member
- [ ] Problem Statement states who is affected and why it matters
- [ ] Goals are measurable or verifiable
- [ ] At least one alternative design was considered and evaluated
- [ ] Trade-offs table has entries — no design is free of trade-offs
- [ ] Architecture has a Mermaid diagram
- [ ] Security Considerations has content or "N/A — [reason]"
- [ ] Operational Considerations has content or "N/A — [reason]"
- [ ] Milestones have concrete deliverables, not vague phases
- [ ] Open Questions are genuine blockers, not deferred decisions
- [ ] Revision Log captures all reviewer feedback and resolutions

If any check fails, fix it before presenting.

### 8. Present for approval

Present the revised document to the user. Highlight:
- Key decisions and their rationale
- Escalated reviewer concerns (Open Questions)
- Discovered work items filed as child tasks

Wait for user feedback. If the user requests changes, revise and re-present until approved.

### 9. Finalize and output

Run `mkdir -p "$COCKPIT_DIR/state/docs"` then write the final document to `$COCKPIT_DIR/state/docs/<slug>.md` where `<slug>` is a kebab-case name (max 50 chars).

Close the epic: `bd close <epic-id>`.

**Output template:**

# Design Doc: [Title]

**Author:** [user name]
**Date:** [YYYY-MM-DD]
**Status:** Draft | Approved

## Context

Background a new team member would need. What exists today. Why we're here.

## Problem Statement

What problem are we solving? Who experiences it? What's the impact of not solving it?

## Goals

Bulleted list of measurable or verifiable goals.

## Anti-Goals

Things this design deliberately does NOT do, and why. Omit only if genuinely no anti-goals.

## Constraints

Hard constraints: compatibility, timeline, resources, regulatory.

## Proposed Design

### Overview

1-2 paragraph summary of the approach.

### Architecture

Mermaid diagram of components and relationships.

### Technical Details

Data models, APIs, algorithms, integration points.

### Data Flow

Step-by-step for key operations. Sequence diagrams where helpful.

## Trade-offs

| Decision | Chosen | Alternative | What We Give Up |
|----------|--------|-------------|-----------------|

## Alternative Designs Considered

### Alternative N: [Name]
- **Approach:** How it would work
- **Pros:** What's good
- **Cons:** Why we didn't choose it

## Security Considerations

Threat model, mitigations. "N/A — [reason]" if not applicable.

## Operational Considerations

Monitoring, rollback, failure modes, capacity. "N/A — [reason]" if not applicable.

## Milestones

| Milestone | Deliverable | Dependencies |
|-----------|-------------|--------------|

## Open Questions

Questions needing human input before implementation.

---

_Metadata_

### Revision Log

| Reviewer | Concern | Resolution |
|----------|---------|------------|

### Tracking

- Epic: <epic-id>
- Child tasks: [list with statuses]

### Next Steps

Optionally run `/design` to create implementation-ready specs, then `/plan` to decompose into tasks.

## Rules

- **Write for humans** — clarity over precision. This will be read by people, not agents.
- **Be opinionated** — make recommendations and defend them. Don't present a menu.
- **Show your work** — for every decision, explain what you rejected and why.
- **Ask questions early** — the gathering phase resolves ambiguity. Don't guess.
- **Diagrams are mandatory** — at least one Mermaid architecture diagram.
- **Track in beads** — epic for the effort, child tasks for discovered work.
- **All reviewers run** — each self-assesses relevance. Don't skip the panel.
- **User approves before finalizing** — the human review step is not optional.
- **Anti-goals matter** — stating what you won't do prevents scope creep.
- **Milestones are concrete** — "Phase 2" is not a milestone. "API returns filtered results" is.
