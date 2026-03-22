You are implementing a design. Your goal is to turn an approved design document into working code using parallel task execution.

## Input

The user has an approved design document (from /design). They may provide:
- A path to the design document
- A reference to the most recent design
- Just "/implement" if the design is in conversation context

If no design is available, stop and tell the user to run /design first.

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Parse task graph | No — main agent | Determines worker decomposition |
| 2. Create shared interfaces | No — main agent | Must exist before workers start |
| 3. Spawn workers | Yes — all workers | Independent implementation tasks |
| 4. Monitor and unblock | No — main agent | Responds to worker issues |
| 5. Integration check | No — main agent | Verifies all pieces connect |
| 6. Self-validation | No — main agent | Checks spec compliance |
| 7. Report | No — main agent | Summarizes results to user |

## Process

### 1. Parse the design into a task graph

Read the design document and extract:
- **Components** to build (from Architecture section)
- **Interfaces** to implement (from Interfaces section)
- **Data schemas** to create/modify (from Data Schemas section)
- **Dependencies** between tasks (which tasks block others)

Produce a task list:
```
Task 1: [description] — depends on: none
Task 2: [description] — depends on: none
Task 3: [description] — depends on: Task 1, Task 2
...
```

Tasks with no dependencies can run in parallel. Maximize parallelism.

### 2. Create the shared interface files first

Before spawning workers, create any shared types, interfaces, or proto definitions that multiple workers need. This prevents workers from inventing incompatible interfaces.

Commit these shared files so workers can read them.

### 3. Spawn all workers (parallel)

Spawn ALL workers in ONE message. Each worker gets:
- Its specific task description
- The relevant interface specs from the design (copy them in, don't reference)
- File paths it should modify or create
- Its dependency list (if it depends on another task, tell it to wait and check)

**Worker prompt template:**
> You are implementing one task from a design.
>
> ## Your Task
> [task description]
>
> ## Interface Spec
> [paste the relevant interfaces from the design]
>
> ## Files
> - Modify: [file paths]
> - Create: [file paths]
>
> ## Dependencies
> [none, or: "Task N must complete first — check that [file] exists before starting"]
>
> ## Rules
> - Follow the interface spec exactly — do not add methods, fields, or parameters not in the spec
> - Match existing code patterns in the repo (error handling, naming, structure)
> - Write tests for the code you write
> - Do not modify files outside your task scope
> - If you're blocked or find the design is ambiguous, report the issue — do not guess

### 4. Monitor and unblock

As workers report back:
- Check that interfaces match the spec
- If a worker is blocked on a dependency, verify the dependency is complete
- If a worker found a design ambiguity, make the call (for minor issues) or flag to the user (for major ones)

### 5. Integration check

After all workers complete:
- Verify all interfaces connect properly
- Run any existing tests (`go test ./...`, `npm test`, `pytest`, etc.)
- Fix integration issues — these are usually import paths, type mismatches, or missing glue code

### 6. Self-validation

Before reporting, verify:

- [ ] Every interface from the design has a corresponding implementation
- [ ] No worker added methods, fields, or parameters not in the spec
- [ ] All tests pass (integration + worker-written unit tests)
- [ ] No files were modified outside declared task scopes
- [ ] Any deviations from the design are documented with rationale

If any check fails, fix it before reporting.

### 7. Report

Report to the user:
- Tasks completed
- Files created/modified
- Test results
- Any issues or deviations from the design

## Rules

- **Do not start without a design** — if there's no design document, stop
- **Shared interfaces first** — create them before spawning workers to prevent drift
- **Workers follow the spec exactly** — no freelancing, no extra methods, no bonus abstractions
- **Match existing patterns** — look at neighboring code for conventions before writing new code
- **Tests are required** — every worker writes tests for its code
- **Report deviations** — if implementation must differ from design, explain why
