# dotfiles

Personal dotfiles for zsh, git, GitHub CLI, Claude Code, and misc tools.

Designed around an **agentic workflow**: encode engineering expertise into composable Claude Code skills that run with minimal human intervention.

## Skill Pipeline

The core workflow is a chain of skills, each a separate phase:

```
/explore  → understand the codebase/system
/design   → team exploration → synthesize → simplify → spec interfaces
/implement → parse design → task graph → parallel workers
/review   → multi-perspective PR review with correctness filter
/pr       → create pull request
```

Two human touchpoints: **approve the design**, **review final findings**. Everything else is automated.

## New Projects

In a greenfield project, use the `/kickoff` skill to bootstrap a project with necessary information.

## Install

```sh
cd ~/jaredonline/dotfiles
./install.sh
```

Creates symlinks from your home directory to the repo. Existing files are backed up with a `.backup` suffix.

## Structure

```
zsh/              — zsh config (.zshrc, .zshprofile, .p10k.zsh)
git/              — git config (.gitconfig)
tools/            — misc tool configs (.gemrc, .terraformrc)
gh/               — GitHub CLI config
claude/           — Claude Code settings + custom commands (skills)
  commands/       — slash command skills (/explore, /design, etc.)
  settings.json   — Claude Code settings (model, plugins, env)
krust/            — Rust CLI wrapper for Claude skills (see krust/README.md)
local/            — GITIGNORED private/work-specific overrides
```

## Conventions

- **New tool = new directory** — each tool's config lives in its own folder
- **All configs support `local/` override** — machine-specific or work-specific settings go in `local/<tool>/`
- **Skills are the product** — iterate on `claude/commands/` like you'd iterate on code

## How I Work with AI

I use Claude Code as my primary development tool. Not as an autocomplete or a chatbot, but as a system of agents that I direct like a tech lead directs a team. The bottleneck in my day isn't writing code. It's identifying what to build, making design decisions, and reviewing output.

My dotfiles repo is the control plane for this system. Skills, rules, and instructions define how agents behave. Improving them is the most impactful work I do on an ongoing basis.

### The Skill Pipeline

I've encoded my development workflow into a chain of Claude Code skills. Each is a slash command that runs a multi-step recipe:

```
/explore  → understand a system or codebase area
/design   → team exploration → synthesize → simplify → spec interfaces
/implement → parse design → task graph → parallel workers
/review   → multi-perspective PR review with correctness filter
/pr       → create pull request
```

Two human touchpoints: I approve the design, and I review the final PR review findings. Everything between is automated.

Each skill is a markdown file that specifies exactly what the agent should do, step by step. They're versioned, composable, and I iterate on them constantly. When a skill produces bad output, I edit the skill. When the agent makes a repeated mistake, I add a rule. The system improves every day.

### Design and Implementation Are Separate Phases

The design phase is where my taste and judgment matter. The agent spawns a team of topic specialists that explore the codebase in parallel, plus a devil's advocate to push back on assumptions. The lead synthesizes their findings into a design document with explicit interface specs: method signatures, request/response types, error cases. Not prose descriptions like "we'll need an endpoint." Actual type definitions.

After the design is drafted, a separate subagent runs a clean-room simplification review. It reads the document with no access to the exploration context and asks: can any interface be removed? Are there unnecessary abstractions? Is any flexibility speculative? This catches about a third of the complexity agents add by default. Agents optimize for completeness. They'll add indirection and abstraction you didn't ask for unless you explicitly check for it.

I review the design document on GitHub (never inline in the terminal), approve or revise, and then implementation runs without me. The agent parses the design into a task graph, spawns parallel workers, and they self-manage dependency ordering. I don't touch it again until the automated review cycle completes.

### Agent Architecture

Every skill uses the same lead/worker pattern:

- The lead decomposes work into parallel units and spawns all workers in a single message (true parallelism, not sequential)
- Each worker gets one focused task and minimal context. Fresh context window, full budget for its specific work.
- The lead aggregates results. It doesn't do the work itself.

I use the best available model (currently Claude Opus) for everything: lead agents, workers, reviewers, explorers. All of it. Speed comes from parallelism: multiple agents running simultaneously, and multiple dev environments running independent workflows. Not from using a faster but weaker model. I don't use smaller models for cost savings. The quality difference between tiers compounds across every agent in the system.

### Context Management

Context is the scarcest resource when working with agents. Claude works best in the first half of its context window. Quality degrades as it fills up: instructions get fuzzy, earlier details get lost, mistakes increase.

I treat context like RAM:
- Pipe verbose command output: `| tail -20` or `| head -50`
- Summarize before forwarding anything between agents. Never pass raw output.
- Delegate to subtasks for anything that reads more than ~5 files or produces more than ~50 lines of output.
- Don't reuse a session that's past 60% context. Start fresh for unrelated work.

Subtasks are the key scaling mechanism. The orchestrator stays lean and holds the plan. Each subtask gets a fresh 200k-token context window for its specific work and returns a short summary. Ten workers each with full context beats one agent trying to hold everything at once.

### Multi-Perspective PR Review

My `/review` skill spawns six reviewer personas in parallel, each with a different focus area:

- **Architect**: structural problems, coupling, boundary violations
- **Code Quality**: readability, error handling, edge cases
- **Devil's Advocate**: failure modes, security, race conditions
- **Prod Readiness**: metrics, logging, graceful degradation
- **SRE/Reliability**: shutdown behavior, retries, load characteristics
- **Test Quality**: tautological tests, mock abuse, missing coverage

After all reviewers return, the lead runs a correctness filter on every finding. It reads the actual code at each referenced location and classifies findings as confirmed, false positive, or partially correct. False positives get removed with justification. When 2+ perspectives flag the same issue, it's treated as a priority.

Language-specific reviewers are spawned conditionally based on what's in the diff. Go files get a Go reviewer (error handling, goroutine leaks). TypeScript files get a TypeScript reviewer (type safety, async patterns). No point spawning reviewers for languages that aren't in the changeset.

### State in Git

All agent output (designs, plans, review findings) goes to a git-tracked state directory. Agents commit and push before asking for my feedback. I review on GitHub where markdown, mermaid diagrams, and tables render properly. I never review large artifacts inline in the CLI.

The state repo clones into every dev environment, so designs from one environment are immediately available in another. No state is locked to a single machine.

### Krust: The Wrapper Harness

[Krust](krust/) is a Rust CLI that wraps Claude Code skills with deterministic orchestration. The core idea: **skills produce content, the harness performs side effects.** A skill writes a markdown artifact and emits JSON action files declaring what should happen (create tasks, archive files). The harness handles everything else: config loading, path resolution, beads task tracking, git commits, tool restrictions, and action processing.

```
krust design nodes "Investigate the disappearances"   # TTRPG node graph
krust design situation "Three guilds competing"        # political situation
krust design code "Add caching layer"                  # software design
krust plan path/to/design.md                           # task decomposition
```

Every subcommand follows the same lifecycle: create a beads task, invoke Claude with restricted tools (git commands are physically disabled, not just discouraged), verify the output, commit and push the draft, then enter an interactive loop where I can approve, give feedback, hand-edit, or quit and resume later. On approve, the harness processes any action files the skill emitted (creating task graphs, archiving consumed inputs) and closes the beads task.

The skill doesn't know about git, beads lifecycle, or path resolution. It sees three env vars (`$KRUST_BEADS_ID`, `$KRUST_OUT`, `$ACTIONS_DIR`) and writes its output. The harness handles the rest. This separation means skills stay focused on their creative or analytical work, and the deterministic parts can't drift across skill implementations.

Resume support means I can quit mid-review, close the terminal, and pick up exactly where I left off with `--resume`. The harness reads the beads task metadata to reconstruct the full context.

### Automation

Once my interactive workflow was stable, I automated the repetitive parts with cron jobs on a dedicated dev environment:

- **CI monitoring** (every 5 minutes): finds CI failures on my open PRs and fixes them automatically. Each PR gets its own git worktree so multiple PRs fix in parallel. Per-PR locking prevents duplicate runs. Cooldown logic skips PRs where a fix was pushed recently.
- **PR review** (every 5 minutes): finds PRs assigned to me for review and runs the multi-perspective review before I look at them. When I open a PR to review, Claude's analysis is already there.
- **Daily report** (8:45am): generates a morning briefing covering task tracker updates, open PR status, git activity across monitored services, Slack mentions, and action items. Filtered by my team, services, and open tasks so it only surfaces what's relevant.

All three run Claude headless (`claude -p "/skill" --dangerously-skip-permissions --output-format stream-json`) and log everything to `~/.local/state/<cron-name>/` with sequential numbering for debugging.

Without automation, my first 45 minutes every morning was spent on CI failures, PR reviews, and getting caught up. Now I start my actual work at 9:15.

### Parallel Workdays

One dev environment = one Claude Code session = one active workflow. I run 5-8 environments simultaneously throughout the day. Some are exploratory (researching an unfamiliar area), some are reviewing PRs, some are actively implementing tasks. I rotate between them as each hits a checkpoint that needs my input.

A typical morning:
```
9:00  Start design on env A
9:05  While A explores, switch to env B, fix a CI issue on another PR
9:15  A has a design draft → review on GitHub
9:20  Approve design on A, it starts implementing
9:25  Switch to env C, start a new PR review
9:30  B finishes CI fix → check, merge
9:35  C has review findings → triage
9:45  A finishes implementation + first review pass → check findings
```

### Writing Voice

I have a `/ghost-write` skill that rewrites text to match my writing style. It loads a persistent style guide, rewrites the input, then spawns a three-reviewer panel in parallel (AI slop detector, tone reviewer, punctuation checker) to catch patterns that don't sound like me. Corporate filler, performative enthusiasm, dramatic pivots, emdashes. One revision pass from the filtered findings, then done.

I use this for docs, PR descriptions, design documents, and anything else that goes to humans. The style guide is the single source of truth for my voice. The reviewers validate against it.

### Delegation

Better briefs produce better agent output. Good delegation means:

- Include file paths, function names, route paths. Anything unique and greppable. Ambiguous terms ("the handler", "the service") send agents searching through the wrong code.
- State the problem and the constraints, not just "fix the bug." What's broken, where it breaks, what the fix needs to preserve.
- Front-load your thinking. If you know the approach, sketch it out. The agent converges faster and the result is better.

Two minutes up front, twenty saved in back-and-forth.

### Iteration

The whole system runs on a simple feedback loop:

1. Run the workflow
2. Observe where it fails or produces poor output
3. Edit the skill or rule that caused the issue
4. Commit the change
5. Run again

Eight weeks in, one command triggers design through review. I review one design doc and one set of findings. Everything else runs without me.

My dotfiles repo is the product. It defines how agents behave, and it gets better every time I fix a skill.

## Private Overrides (`local/`)

The `local/` directory is gitignored and holds machine-specific or work-specific config:

- `local/zsh/.zshrc.local` — sourced at the end of `.zshrc`
- `local/zsh/.zshprofile.local` — sourced at the end of `.zshprofile`
- `local/git/.gitconfig.local` — included by `.gitconfig` via `[include]`
- `local/claude/settings.json` — Claude Code overrides (merged via `jq`)
- `local/install.sh` — additional symlink mappings (sourced by main `install.sh`)

To set up local overrides, create the files in `local/` and re-run `./install.sh`.
