You are investigating a question by gathering evidence from multiple sources. Your goal is to answer the question with cited evidence organized by sub-question.

## Input

The user provides a question to investigate via ARGUMENTS. Examples:
- "audit all LaunchDarkly clients in the codebase"
- "can we split the LD relay proxy into event and update pools"
- "what logging do we have for the payments service"

If no question is provided, stop and ask the user for a question.

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Create Beads task | No — main agent | Track work before starting |
| 2. Decompose question | No — main agent | Break into 3-5 sub-questions with search terms |
| 3. Spawn explorers (Code Explorer, External Explorer) | Yes — 2 agents | Independent source types |
| 4. Synthesize | No — main agent | Groups findings by sub-question |
| 5. Self-validate | No — main agent | Checks completeness and sourcing |
| 6. Write report + close Beads | No — main agent | Outputs investigation and closes tracking |

## Process

### 1. Create Beads task

Run `bd create --title="Investigate: [question]" --type=task` and store the returned task ID.
Claim it: `bd update <id> --claim`.

### 2. Decompose question

Break the question into 3-5 sub-questions. For each sub-question, identify:
- What specifically needs to be answered
- Relevant search terms (function names, config keys, service names, etc.)

Cap at 5 sub-questions. If the question is narrow enough, fewer is better.

### 3. Spawn explorers (parallel)

Spawn BOTH explorers in ONE message.

**Code Explorer** (Agent, model=opus):
> You are searching the codebase to answer specific sub-questions with evidence.
>
> ## Sub-Questions
> [paste the sub-questions and search terms]
>
> ## Output Format
> For each finding, return:
> - Sub-question: which sub-question this answers
> - Finding: what was found
> - Source: file:line
>
> Search broadly — use Grep, Glob, and Read to find evidence. Follow references across files. If a sub-question has no relevant code, say so explicitly.

**External Explorer** (Agent, model=opus):
> You are searching external sources (Slack, GitHub, web) to answer specific sub-questions with evidence.
>
> ## Sub-Questions
> [paste the sub-questions and search terms]
>
> ## Output Format
> For each finding, return:
> - Sub-question: which sub-question this answers
> - Finding: what was found
> - Source: URL (Slack message, GitHub PR/issue, web page)
> - Date: when the source was created/last modified
>
> Search Slack, GitHub issues/PRs, and web documentation. If a sub-question has no relevant external evidence, say so explicitly.

### 4. Synthesize

Group findings from both explorers by sub-question. For each sub-question:
- Write a direct answer
- Assign confidence: **High** (multiple corroborating sources), **Medium** (single source or partial evidence), **Low** (inferred or uncertain)
- List supporting evidence with source citations
- When code evidence contradicts external sources, code wins

### 5. Self-validate

Before writing the report, verify:

- [ ] Every finding cites a source with `file:line` or URL with date
- [ ] Every sub-question has an answer (even if "Could not determine — [reason]")
- [ ] Code evidence takes precedence over external sources when conflicting
- [ ] Sub-questions are capped at 5
- [ ] Beads task ID is available for the Tracking section

If any check fails, fix the gap before proceeding.

### 6. Write report + close Beads

Ensure the output directory exists: `mkdir -p $COCKPIT_DIR/state/investigations`

Close the beads task: `bd close <task-id>`

Write the report to `$COCKPIT_DIR/state/investigations/<slug>.md` where `<slug>` is a kebab-case version of the question (max 50 chars).

Report template:

```markdown
# Investigation: [Question]

## TL;DR
2-3 sentence answer to the question.

## Sub-Questions & Findings

### Q1: [sub-question]
**Answer**: [direct answer]
**Confidence**: High/Medium/Low

**Evidence**:
- description — `file:line`
- description — [Slack, YYYY-MM-DD](url)
- description — [PR #123](url)

### Q2: [sub-question]
...

## Recommendations
Numbered next steps based on findings.

## Sources
Full list of all sources consulted, with dates where available.

## Tracking
- Beads: <task-id> — closed
```

## Rules

- Every finding must cite a source — no unsourced claims
- Every sub-question must have an answer, even if "Could not determine — [reason]"
- Code evidence takes precedence over external sources when they conflict
- Sub-questions are capped at 5 per investigation
- Report must be written to `$COCKPIT_DIR/state/investigations/<slug>.md`
- Beads task must be created before exploration and closed before writing the report
- All agents use model=opus
- Spawn both explorers in ONE message — parallel, not sequential
