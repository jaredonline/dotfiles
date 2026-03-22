You are bootstrapping a new project. Your job is to create 4 foundational files and set up task tracking so both humans and LLMs can work effectively in this codebase.

## Artifacts to Create

1. **README.md** — For humans. Project name, what it does, how to install/run, how to contribute.
2. **ARCHITECTURE.md** — For LLMs. High-level system design, key components, data flow, directory structure.
3. **PRINCIPLES.md** — For LLMs. Design principles, coding conventions, decisions and their rationale.
4. **DESIGN.md** — For LLMs. Visual design elements and standards for the project. Not valid for all projects
5. **Task tracking** — For both. Living task list. Updated after every step.
   - If `bd` is available (`command -v bd` succeeds) and initialized in the repo (`bd list` succeeds), use `bd` for all task management. Create tasks with `bd add`, update status with `bd update`, and check progress with `bd list`.
   - If `bd` is not available or not initialized, ask the user:
     > How do you want to track tasks for this project?
     > 1. **TODOS.md** — Simple markdown checklist (`- [ ] task` / `- [x] done`)
     > 2. **GitHub Issues** — Track tasks as issues (requires `gh` CLI)
     > 3. **Something else** — Describe your preferred method and I'll adapt
   - If the user picks option 3, ask them to describe their method. Record it and use it consistently whenever updating task tracking in step 2.

## Process

1. **Ask first.** Before creating anything, ask the user:
   - What is this project? What problem does it solve?
   - What language/framework/tools are you using?
   - Is there existing code to examine?
   - Any strong opinions on architecture or conventions?
   - How do you want to organize this project? Offer three paths:
     > 1. **I have a method** — Describe your preferred project organization (directory structure, naming, module boundaries) and I'll follow it
     > 2. **Suggest something** — I'll propose an organization based on the language/framework and project type
     > 3. **Let's talk it through** — We'll discuss trade-offs together before committing to a structure
   - If the user picks option 3, discuss until you converge on a directory structure and module boundaries. Summarize the agreed approach in a short list and get confirmation before proceeding.

2. **Create files one at a time.** After each file:
   - Update task tracking to reflect progress (using whichever method was selected in step 1)
   - Verify all files exist (create empty placeholders for ones not yet written)
   - Stop and ask: "Ready for the next file?"

3. **Start with task tracking** (skeleton tasks for each file to create), then README.md, then ARCHITECTURE.md, PRINCIPLES.md, and DESIGN.md last.

4. **If existing code exists**, read it thoroughly before writing ARCHITECTURE.md and PRINCIPLES.md — derive conventions from what's already there rather than inventing new ones.

## Guidelines

- Keep files concise. Prefer bullet points over paragraphs.
- ARCHITECTURE.md should include a directory tree if the project has structure.
- PRINCIPLES.md should explain *why*, not just *what*.
- DESIGN.md is where trade-offs and "why not X" explanations live.
