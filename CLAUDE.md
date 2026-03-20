# Claude Code — Global Instructions

## Project Context

This is a dotfiles repo that doubles as an agentic workflow system. Config files are symlinked to their expected locations. Claude Code skills encode a multi-phase development pipeline.

Read ARCHITECTURE.md for structure and PRINCIPLES.md for conventions before making changes.

## Skill Pipeline

The core workflow: `/explore` → `/design` → `/implement` → `/review` → `/pr`

Two human touchpoints: approve the design, review final findings. Everything between is automated.

## Agent Behavior

- **Use opus for all agents** — lead, workers, reviewers, explorers. No exceptions.
- **Lead/worker pattern** — decompose into parallel units, spawn all workers in ONE message, aggregate results.
- **Context hygiene** — pipe verbose output (`| tail -20`), summarize before forwarding to subagents, delegate to subtasks for anything reading >5 files.
- **Don't over-engineer** — no speculative abstractions, no "just in case" flexibility. Build what's needed, nothing more.
- **Spec interfaces explicitly** — method signatures, request/response types, error cases. Not prose descriptions.

## Repo Conventions

- New tool = new directory at repo root
- All configs support `local/` override (gitignored)
- `install.sh` is the single entry point for setup — symlinks and JSON merges
- Skills live in `claude/commands/` — edit them like code, iterate constantly

## Skill Quality Rules

When editing skills in `claude/commands/`:
- Integrate new behavior into existing steps — don't append standalone sections at the bottom
- Consolidate shared structure — don't duplicate guidance across sections
- Order sections by workflow order, not by when they were written
- Read the skill top-to-bottom after every edit to verify cohesion

## Review Standards

- Every finding must be verified against actual code before reporting
- Classify findings: Confirmed | False positive | Partially correct
- Remove false positives, log removals with justification
- When 2+ reviewer perspectives flag the same issue, it's a priority

## What NOT to Do

- Don't ask unnecessary confirmation questions mid-workflow — if the skill specifies the steps, follow them
- Don't use weaker models for "speed" — parallelism provides speed
- Don't review large artifacts inline in the CLI — push to git, review on GitHub
- Don't forward raw output between agents — always summarize first
