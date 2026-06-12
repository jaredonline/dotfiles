---
name: commit-message
description: Generate a commit message from a diff file. Called by `/pr` (style=pr, sub-agent).
user_invocable: false
---

You are generating a single commit message from a precomputed diff. Your only output is the message text on stdout.

## Inputs

Inputs come from environment variables set by the caller:

- `COMMIT_DIFF_PATH` (required) — absolute path to a file containing the diff. The file holds a unified diff (`git diff HEAD`), optionally followed by an `[untracked]` block listing new files one per line.
- `COMMIT_MSG_STYLE` (optional, default `ship`) — one of `ship` or `pr`. If unset or empty, treat as `ship`.

## Tool restrictions

This skill MUST NOT invoke `git`, `gh`, or `gt`. The only allowed tool is `Read` (to load the diff file). When spawned as a sub-agent by `/pr`, do not shell out for diffs or repository state; read only from `$COMMIT_DIFF_PATH`.

## Invocation pattern

- **Sub-agent (from `/pr`):** `/pr` writes the diff to a temp file and spawns `/commit-message` via the Agent tool with prompt-level instructions specifying the diff path and `style=pr`. The sub-agent parses the final stdout as the message.

## Process

1. Read the diff from `$COMMIT_DIFF_PATH`.
2. If the diff content is empty (zero bytes or only whitespace), print an error message to stdout explaining that the diff is empty and exit.
3. Determine the style: read `$COMMIT_MSG_STYLE`; if unset or empty, use `ship`.
4. Generate a message per style:
   - **`ship`** — lowercase imperative subject summarizing WHAT changed, blank line, then a 1–3 sentence body explaining WHY the change was made. Subject ≤72 characters. No `[area]` prefix. No trailing period on the subject.
   - **`pr`** — subject-only, formatted as `[area1] [area2] lowercase description`. Areas are short tags derived from touched components (for example `[api]`, `[skills]`, `[ci]`). Subject ≤72 characters including the tags. No body.
5. Self-validate the generated message before printing:
   - The subject (first non-empty line) is ≤72 characters.
   - None of these refusal phrases appear anywhere in the message: `I can't`, `I cannot`, `I'm unable`, `As an AI`, `Sorry,`.
   - No triple-backtick code fences.
   - No markdown headers (lines starting with `#`).
   - No commentary, preamble, or explanation around the message — just the message itself.
6. If self-validation fails, regenerate once. If the second attempt also fails validation, print an error to stdout describing which check failed and exit.
7. On success, print the final message to stdout as the agent's final response so the `/pr` caller can parse it.

## Rules

- **No git, gh, or gt** — the caller owns all VCS work. Use only `Read`.
- **Read the diff from `$COMMIT_DIFF_PATH`** — never shell out to recompute it.
- **No filesystem writes** — emit the message on stdout only.
- **One message per invocation** — do not emit multiple candidates or commentary.
- **Respect the style** — `ship` always has a body; `pr` never does. Never mix formats.
- **Stdout is authoritative for sub-agents** — the final agent response must be exactly the message, with no surrounding text.
