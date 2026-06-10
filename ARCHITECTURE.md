# Architecture

## Overview

A dotfiles repo that doubles as an **agentic workflow system**. Config files are symlinked to their expected locations. Claude Code skills encode a multi-phase development pipeline that runs with minimal human intervention.

## Directory Structure

```
dotfiles/
├── install.sh              # Symlinks + JSON merges, entry point for setup
├── CLAUDE.md               # Global agent instructions (TODO)
├── ARCHITECTURE.md         # This file
├── PRINCIPLES.md           # Design principles and conventions
├── TODOS.md                # Living task list
│
├── claude/                 # Claude Code configuration
│   ├── settings.json       # Model, plugins, env vars
│   └── commands/           # Skills (slash commands) + graph pipelines
│       ├── kickoff.md      # Project bootstrapping
│       ├── explore.dot     # Explore pipeline graph (run via `krust explore`)
│       ├── prompts/        # Per-node prompt files for graph pipelines
│       ├── design.md       # Architecture + interface spec
│       ├── implement.md    # Parallel task execution
│       ├── review.md       # Multi-perspective PR review
│       └── pr.md           # Pull request creation
│
├── zsh/                    # Shell configuration
│   ├── .zshrc              # Main shell config (oh-my-zsh, plugins, PATH)
│   ├── .zshprofile         # Login shell profile
│   └── .p10k.zsh           # Powerlevel10k prompt theme
│
├── git/                    # Git configuration
│   └── .gitconfig          # User, LFS, includes .gitconfig.local
│
├── gh/                     # GitHub CLI
│   └── config.yml          # Protocol, aliases, display settings
│
├── tools/                  # Miscellaneous tool configs
│   ├── .gemrc              # Ruby gems
│   └── .terraformrc        # Terraform
│
└── local/                  # GITIGNORED — machine-specific overrides
    ├── install.sh          # Extra symlinks for this machine
    ├── zsh/
    ├── git/
    └── claude/
```

## Install Flow

`install.sh` does two things:

1. **Symlinks** — `link_file` creates `~/.foo → dotfiles/tool/.foo` with backup of existing files
2. **JSON merges** — `merge_json` deep-merges base + `local/` override via `jq -s '.[0] * .[1]'` (used for Claude settings)

Then sources `local/install.sh` if present for machine-specific additions.

## Skill Pipeline

The skills form a linear pipeline. Each phase has different properties:

```
krust explore ──→ /design ──→ /implement ──→ /review ──→ /pr
  │                 │            │              │           │
  │                 │            │              │           └─ Create PR
  │                 │            │              └─ Multi-perspective review
  │                 │            └─ Parallel task workers
  │                 └─ Team exploration → synthesize → simplify
  └─ Document/understand a system
```

### krust explore — Understand
- **Purpose**: Build a mental model of an unfamiliar system
- **Pattern**: A DOT graph (`claude/commands/explore.dot`) executed by the krust graph engine — parallel topic explorers fan out, a fan-in indexes their findings, a synthesizer writes the artifact, and a hexagon human gate pauses the run (exit 75) until `krust graph resume <id> --gate-choice ...`. The retired `/explore` slash command is replaced by this CLI entry point.
- **Output**: Markdown document with request flows, key invariants, data stores, integration points, architecture diagrams
- **Human role**: Read the output to build context, then approve or iterate at the feedback gate

### /design — Decide
- **Purpose**: Produce an architecture document with specced interfaces
- **Pattern**: Lead spawns topic specialists + devil's advocate → synthesize → clean-room simplification review
- **Output**: Design document with API specs, interface definitions, data flows, invariants, mermaid diagrams
- **Human role**: **Review and approve** (primary touchpoint)
- **Key rule**: Spec interfaces explicitly (method signatures, request/response types, error cases), not just prose descriptions

### /implement — Build
- **Purpose**: Turn an approved design into code
- **Pattern**: Parse design into task graph with dependencies → spawn all workers simultaneously → workers self-manage dependency waiting → lead monitors
- **Output**: Code changes on a branch
- **Human role**: None until review

### /review — Verify
- **Purpose**: Multi-perspective code review with correctness filtering
- **Pattern**: Lead spawns reviewer personas (architect, code quality, devil's advocate, prod readiness, SRE, test quality) → each reviews in parallel → lead aggregates, deduplicates, runs correctness filter on each finding → fix cycle → present to human
- **Output**: Structured review findings by severity, false positives removed
- **Human role**: **Review final findings** (second touchpoint)

### /pr — Ship
- **Purpose**: Create a pull request with structured description
- **Output**: GitHub PR

## Agent Architecture

All skills use the **lead/worker pattern**:

```
Lead (flagship):
  - Decomposes work into parallel units
  - Spawns all workers in ONE message (true parallelism)
  - Aggregates results, doesn't do the work itself

Workers (flagship):
  - Each gets one focused task + minimal context
  - Fresh context window, full budget for their work
  - Reports back to lead when done
```

Flagship everywhere — `[models].flagship` in the krust config defines which model that is. Speed comes from parallelism, not faster models.

## Local Override System

Every tool directory has a corresponding `local/<tool>/` directory. The pattern:

| Tool | Base | Override | Mechanism |
|------|------|----------|-----------|
| zsh | `zsh/.zshrc` | `local/zsh/.zshrc.local` | `source` at end of .zshrc |
| git | `git/.gitconfig` | `local/git/.gitconfig.local` | `[include] path` |
| claude | `claude/settings.json` | `local/claude/settings.json` | `jq` deep merge |
| install | `install.sh` | `local/install.sh` | `source` at end |
