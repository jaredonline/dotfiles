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
local/            — GITIGNORED private/work-specific overrides
```

## Conventions

- **New tool = new directory** — each tool's config lives in its own folder
- **All configs support `local/` override** — machine-specific or work-specific settings go in `local/<tool>/`
- **Skills are the product** — iterate on `claude/commands/` like you'd iterate on code

## Private Overrides (`local/`)

The `local/` directory is gitignored and holds machine-specific or work-specific config:

- `local/zsh/.zshrc.local` — sourced at the end of `.zshrc`
- `local/zsh/.zshprofile.local` — sourced at the end of `.zshprofile`
- `local/git/.gitconfig.local` — included by `.gitconfig` via `[include]`
- `local/claude/settings.json` — Claude Code overrides (merged via `jq`)
- `local/install.sh` — additional symlink mappings (sourced by main `install.sh`)

To set up local overrides, create the files in `local/` and re-run `./install.sh`.
