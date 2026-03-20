You are bootstrapping a new project. Your job is to create 5 foundational files that give both humans and LLMs the context they need to work effectively in this codebase.

## Files to Create

1. **README.md** — For humans. Project name, what it does, how to install/run, how to contribute.
2. **ARCHITECTURE.md** — For LLMs. High-level system design, key components, data flow, directory structure.
3. **PRINCIPLES.md** — For LLMs. Design principles, coding conventions, decisions and their rationale.
4. **DESIGN.md** — For LLMs. Visual design elements and standards for the project. Not valid for all projects
5. **TODOS.md** — For both. Living task list. Updated after every step.

## Process

1. **Ask first.** Before creating anything, ask the user:
   - What is this project? What problem does it solve?
   - What language/framework/tools are you using?
   - Is there existing code to examine?
   - Any strong opinions on architecture or conventions?

2. **Create files one at a time.** After each file:
   - Update TODOS.md to reflect progress
   - Verify all 5 files exist (create empty placeholders for ones not yet written)
   - Stop and ask: "Ready for the next file?"

3. **Start with TODOS.md** (as a skeleton), then README.md, then ARCHITECTURE.md, PRINCIPLES.md, and DESIGN.md last.

4. **If existing code exists**, read it thoroughly before writing ARCHITECTURE.md and PRINCIPLES.md — derive conventions from what's already there rather than inventing new ones.

## Guidelines

- Keep files concise. Prefer bullet points over paragraphs.
- ARCHITECTURE.md should include a directory tree if the project has structure.
- PRINCIPLES.md should explain *why*, not just *what*.
- DESIGN.md is where trade-offs and "why not X" explanations live.
- TODOS.md uses checkbox format: `- [ ] task` / `- [x] done`.
