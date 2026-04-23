---
name: commit-message
description: Generate a commit message from a diff file. Called by `krust ship` (style=ship) and `/pr` (style=pr, sub-agent).
user_invocable: false
---

You are generating a single commit message from a precomputed diff. Your only outputs are the message text (stdout) and, if `$KRUST_OUT` is set, the same message written to that file.

## Inputs

Inputs come from environment variables set by the caller:

- `KRUST_COMMIT_DIFF_PATH` (required) — absolute path to a file containing the diff. The file holds a unified diff (`git diff HEAD`), optionally followed by an `[untracked]` block listing new files one per line.
- `KRUST_COMMIT_MSG_STYLE` (optional, default `ship`) — one of `ship` or `pr`. If unset or empty, treat as `ship`.
- `KRUST_OUT` (required on the standalone path; may be unset on the sub-agent path) — absolute path where the final message is written. If unset, skip the file write and rely on stdout.

## Tool restrictions

This skill MUST NOT invoke `git`, `gh`, or `gt`. The only allowed tools are `Read` (to load the diff file) and `Write` (to emit `$KRUST_OUT`). Krust enforces this via `--allowedTools "Read" "Write"` and `--disallowedTools "Bash(git:*)" "Bash(gh:*)" "Bash(gt:*)"`. When spawned as a sub-agent by `/pr`, the same restrictions apply — do not shell out for diffs or repository state; read only from `$KRUST_COMMIT_DIFF_PATH`.

## Invocation patterns

- **Standalone (from `krust ship`):** Krust writes the diff to a temp file, exports `KRUST_COMMIT_DIFF_PATH`, `KRUST_COMMIT_MSG_STYLE`, and `KRUST_OUT`, then runs `claude -p /commit-message`. The skill reads the diff, generates a message, writes it to `$KRUST_OUT`, and prints it to stdout.
- **Sub-agent (from `/pr`):** `/pr` writes the diff to a temp file and spawns `/commit-message` via the Agent tool with prompt-level instructions specifying the diff path and `style=pr`. `$KRUST_OUT` may be unset; the sub-agent parses the final stdout as the message. Behavior is otherwise identical.

## Process

1. Read the diff from `$KRUST_COMMIT_DIFF_PATH`.
2. If the diff content is empty (zero bytes or only whitespace), print an error message to stdout explaining that the diff is empty and exit WITHOUT writing `$KRUST_OUT`.
3. Determine the style: read `$KRUST_COMMIT_MSG_STYLE`; if unset or empty, use `ship`.
4. Generate a message per style:
   - **`ship`** — lowercase imperative subject summarizing WHAT changed, blank line, then a 1–3 sentence body explaining WHY the change was made. Subject ≤72 characters. No `[area]` prefix. No trailing period on the subject.
   - **`pr`** — subject-only, formatted as `[area1] [area2] lowercase description`. Areas are short tags derived from touched components (for example `[krust]`, `[skills]`, `[ci]`). Subject ≤72 characters including the tags. No body.
5. Self-validate the generated message before writing:
   - The subject (first non-empty line) is ≤72 characters.
   - None of these refusal phrases appear anywhere in the message: `I can't`, `I cannot`, `I'm unable`, `As an AI`, `Sorry,`.
   - No triple-backtick code fences.
   - No markdown headers (lines starting with `#`).
   - No commentary, preamble, or explanation around the message — just the message itself.
6. If self-validation fails, regenerate once. If the second attempt also fails validation, print an error to stdout describing which check failed and exit WITHOUT writing `$KRUST_OUT`.
7. On success, if `$KRUST_OUT` is set, write the final message to `$KRUST_OUT` in a single write (no partial files). Always print the final message to stdout as the agent's final response so sub-agent callers that do not set `$KRUST_OUT` can parse it.

## Rules

- **No git, gh, or gt** — the caller blocks these tools and owns all VCS work. Use only `Read` and `Write`.
- **Read the diff from `$KRUST_COMMIT_DIFF_PATH`** — never shell out to recompute it.
- **Write only to `$KRUST_OUT`** (when set) — no other filesystem writes.
- **One message per invocation** — do not emit multiple candidates or commentary.
- **Respect the style** — `ship` always has a body; `pr` never does. Never mix formats.
- **Stdout is authoritative for sub-agents** — the final agent response must be exactly the message, with no surrounding text.
