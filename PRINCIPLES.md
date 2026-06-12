# Principles

## Core Philosophy

**Your dotfiles are the product.** Skills, rules, and CLAUDE.md files define agent behavior. Improving them is the highest-leverage work you can do.

## Design Principles

### Command-Driven, Not Discovery-Driven

Don't type open-ended requests and hope Claude figures it out. Invoke specific skills that encode your workflow. Each skill is a precise, multi-step recipe — reproducible, debuggable, composable, improvable.

**Why:** Ad-hoc prompts don't scale. They work for simple tasks but fall apart for anything multi-step. Skills are how you encode expertise into a system that runs without you.

### Separate Design from Implementation

Design and implementation are fundamentally different phases with different properties. Design requires broad codebase exploration and human review. Implementation requires narrow task focus and parallel execution.

**Why:** The design is where your taste shows up. Review the design once, then let automation handle the rest until the final review. Your time is the bottleneck — spend it on design decisions, not catching bugs that automated review will find.

### Spec Interfaces, Not Implementation

Design documents must explicitly define all APIs and key interfaces — method signatures, request/response types, error cases. Prose descriptions ("we'll need an endpoint") are not enough.

**Why:** When parallel workers implement different components simultaneously, the interface spec is their contract. Without it, they'll each invent their own. The API is where your taste shows up — let agents fill in implementation details.

### Simplify Aggressively

Agents over-engineer by default. They add abstractions, indirection, and flexibility you didn't ask for. Every design gets a clean-room simplification review.

**Why:** Agents optimize for completeness, not simplicity. The simplification review catches the ~30% of complexity added "just in case." Build this into skills, don't rely on catching it during code review.

### Opus Everywhere

Use Opus for everything — lead, workers, reviewers, explorers. Speed comes from parallelism (multiple agents, multiple devboxes), not from faster models.

**Why:** The quality difference between model tiers is massive and compounds across every agent in your workflow. Don't use a weaker model "to save cost" unless you've actually A/B tested it and measured the quality reduction.

## Skill Design Conventions

### Lead/Worker Pattern for Everything

Every skill decomposes work into parallel units using the same pattern:

- Lead decomposes and spawns all workers in ONE message (true parallelism)
- Workers each get one focused task + minimal context
- Lead aggregates results, doesn't do the work itself

**Why:** 10 flagship agents in parallel are faster *and* better than 10 budget-tier agents. Subagents also provide fresh context windows — the fundamental scaling primitive.

### Context Is the Scarcest Resource

- Pipe verbose commands: `| tail -20` or `| head -50`
- Summarize before forwarding to subagents
- Delegate to subtasks for anything reading >5 files or producing >50 lines
- Stay in the top half of the context window; quality degrades past 50%

**Why:** Claude works best in the first half of the context window. Instructions get fuzzy, earlier details get lost, and mistakes increase as context fills up.

### Echo the Brief

The brief surfaces in two places:

- **Console**: the skill prints the brief italicized on a single line before spawning agents, so the brief survives any subsequent failure into the terminal scrollback.
- **Artifact**: skills that produce a markdown artifact append a `## Brief` section after `## Tracking` (and `## Next Step` if present) and before any `## Rounds of Feedback`. Format is a blockquote (`> `) with the brief preserved verbatim, newlines included.

If the brief is empty, skip the console echo and omit `## Brief`. Path-based skills (taking a path, diff, or branch) are exempt. `feedback-*` skills preserve `## Brief` byte-for-byte during revision, same as `## Rounds of Feedback`.

**Why:** The brief is the single source of truth for what the human asked for. Echoing it makes intent auditable in transcripts and lets downstream skills (and humans) recover the original ask without re-reading the parent's context.

### Protect Skill Quality as You Iterate

Skills degrade when new features are appended rather than woven into existing structure. Every edit must preserve cohesion:

- Integrate into existing steps, don't append standalone sections
- Consolidate shared structure, don't duplicate guidance
- Order sections by workflow order, not by when they were written
- Read top-to-bottom after editing

**Why:** Without this discipline, skills accumulate cruft over time and become unreliable.

## Dotfiles Conventions

### New Tool = New Directory

Each tool's config lives in its own folder at the repo root. Don't mix configs.

**Why:** Clean separation makes it easy to add/remove tools and keeps the install script straightforward.

### All Configs Support Local Override

Every tool directory has a corresponding `local/<tool>/` directory for machine-specific settings. The `local/` directory is gitignored.

**Why:** The same dotfiles repo targets both local macOS machines and AI-managed devboxes. Machine-specific settings (tokens, paths, work-specific aliases) shouldn't be committed.

### Remove Friction Relentlessly

Every time you find yourself:
- Answering "yes" to a confirmation → remove the confirmation
- Correcting the same mistake → add a rule
- Running two commands in sequence → combine them into a skill

**Why:** The goal is one command, zero interruptions, correct output. Every friction point you remove compounds over time.

## Iteration Process

1. Run the workflow
2. Observe where it fails or produces poor output
3. Edit the skill/rule that caused the issue
4. Commit the dotfile change
5. Run again

The system gets better every time you improve a dotfile. A bad skill you improve daily beats a perfect plan you never build.
