You are running an on-call support loop. Your goal is to monitor Slack channels for unanswered questions and Datadog for alert changes, research issues, and draft responses for the on-call engineer.

## Input

Invocation:

```
/oncall-support-loop [#channel ...] [--monitors=FILTER] [--team=TAG]
```

Parse ARGUMENTS to extract three values. For each, resolve in this order — ARGUMENTS first, then the corresponding env var. There are no hardcoded literal defaults:

| Value | ARGUMENTS token | Env var fallback | Required? |
|---|---|---|---|
| `slack_channels` | all tokens starting with `#` | `$ONCALL_SLACK_CHANNELS` (space-separated list) | **Required** |
| `monitor_filter` | value after `--monitors=` | `$ONCALL_MONITOR_FILTER` | Optional — empty means skip Datadog entirely |
| `monitor_team_tag` | value after `--team=` | `$ONCALL_MONITOR_TEAM_TAG` | Required **iff** `monitor_filter` is non-empty |

If a required value cannot be resolved, STOP immediately with an actionable error naming the missing env var. Examples:

- No `#channel` in ARGUMENTS and `$ONCALL_SLACK_CHANNELS` is unset → stop with: `error: no Slack channels provided. Pass #channel in ARGUMENTS or set $ONCALL_SLACK_CHANNELS.`
- `monitor_filter` resolved but `monitor_team_tag` unresolved → stop with: `error: monitor filter set but no team tag. Pass --team=TAG or set $ONCALL_MONITOR_TEAM_TAG.`

If `monitor_filter` is empty after resolution, skip the entire Datadog flow (Section 4 baseline, Datadog cron, Datadog loop iteration). The skill runs Slack-only in that mode.

### Datadog API quirks

- The Datadog `get_monitors` API only supports name-based filtering. Tag-based filtering times out. So we query by name, then filter results client-side to monitors tagged with `$monitor_team_tag`.
- **Tag casing caveat**: some monitors are double-tagged with both kebab-case and underscore forms (e.g., `team:foo-bar` AND `team:foo_bar`). Checking one is enough; the kebab-case is the canonical form. Match the casing of `$monitor_team_tag` as provided.

### REPL fallback (no krust)

The skill is also invocable as `/oncall-support-loop` directly, outside the krust lifecycle. Detect this by checking whether `$ACTIONS_DIR` and `$KRUST_OUT_DIR` are both set:

- **Under krust** (both set): run the full flow — write scan summary, generate slug, compute final path, create crons, persist bd metadata, write briefing file, emit artifact.
- **REPL fallback** (either unset): skip the scan-summary write, slug generation, final-path compute, bd metadata persist, briefing-file write, and artifact emit. Still create the crons (names use an inline timestamp slug like `oncall-YYYYMMDD-HHMM-<type>`), and present the briefing inline in the response.

Every step below that writes to `$ACTIONS_DIR` or `$KRUST_OUT_DIR`, calls `bd update $KRUST_BEADS_ID`, or shells out to `krust` is guarded by this check.

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Initial scan | No | Establish baseline of existing questions and alert states |
| 2. Research open questions | Yes | One /explore agent per unanswered question |
| 3. Ghost-write responses | No | Depends on research results |
| 4. Baseline alerts | No | Query Datadog monitors to establish known alert set (skipped if `monitor_filter` empty) |
| 5. Scan summary + slug + final path | No | Deterministic file prep for krust artifact |
| 6. Start loop (crons) | No | Set up cron jobs for ongoing monitoring |
| 7. Persist metadata + briefing + artifact | No | Hand output back to krust |

## Process

### 1. Initial Slack scan

For each channel in `$slack_channels`:
1. Fetch the channel using `mcp__slack__fetch` with `#channel-name`
2. Search recent messages with `mcp__slack__search` for `in:#channel-name` (limit 30)
3. Fetch individual threads for messages that look like questions
4. Classify each thread as:
   - **Unanswered**: question with no replies, or question where the last message is still the asker
   - **Answered**: has substantive replies from others
   - **In progress**: someone acknowledged but hasn't answered yet (e.g., "looking into it")

Build a list of unanswered questions. For each, record:
- Thread URL
- Author
- One-line summary of the question
- Timestamp

Present the list to the user: "Found N unanswered questions. Researching now."

### 2. Research open questions (parallel)

For each unanswered question, spawn an /explore agent:

**Question Explorer** (Agent, subagent_type=Explore, model=opus):
> Thoroughness: very thorough
>
> I'm on call. Someone asked in Slack:
>
> "[paste the question text]"
>
> I need to research this. [Include specific guidance on what to search for based on the question content.]
>
> Report what you find with specific file paths and line numbers.

Spawn ALL explorers in ONE message for parallel execution.

After all explorers complete, summarize findings for each question with:
- The answer (verified with file paths and line numbers)
- Key caveats or uncertainties

### 3. Ghost-write responses

For each researched question, invoke `/ghost-write` with the key points to cover. Present the draft response to the user.

Do NOT post any Slack messages. The user will copy-paste responses manually.

### 4. Baseline Datadog alerts

**Skip this entire section if `$monitor_filter` is empty.**

1. Call `mcp__datadog__get_monitors` with `name=$monitor_filter`. The response is always too large (~615K chars for a typical team-scoped query) and will be saved to a file rather than returned inline. That's expected — do not retry expecting a smaller response.
2. Parse the saved file with python. The wrapper is `[{"type": "text", "text": "..."}]` and the `text` field has a `Monitors: ` prefix before the JSON array, which must be stripped:
   ```python
   import json, os, re
   team_tag = os.environ["ONCALL_MONITOR_TEAM_TAG"]  # or pass in via the command line
   with open(PATH) as f: wrapper = json.load(f)
   text = wrapper[0]["text"]
   m = re.search(r'Monitors:\s*(\[.*\])\s*$', text, re.DOTALL)
   monitors = json.loads(m.group(1))
   team = [x for x in monitors if team_tag in x.get("tags", [])]
   alerting = sorted(x["id"] for x in team if x.get("status") in ("Alert", "Warn"))
   nodata = sorted(x["id"] for x in team if x.get("status") == "No Data")
   ```
   Status values are exact capitalized strings: `"Alert"`, `"Warn"`, `"No Data"`, `"OK"`.
3. Store two baseline sets: `alerting` (Alert/Warn) and `nodata` (No Data). Track No Data separately because it often represents chronic gaps (broken metric pipelines, decommissioned services) that will never recover without intervention and shouldn't pollute the Alert diff.
4. For each alerting monitor, launch an /explore agent to investigate:
   - What metric/service is alerting
   - Whether it's chronic/known or new (flapping alerts around their threshold are common — note them as chronic so iteration reports stay quiet)
   - Recent code changes that could be related
   - Whether it's actionable
5. Present the alert summary to the user, including which alerts are flagged chronic.

**Transient API errors**: both `mcp__datadog__get_monitors` (ECONNRESET) and `mcp__slack__search` (502) fail transiently in normal operation. Retry once silently. If the retry also fails, report it and skip the iteration — do not block.

### 5. Write scan summary

**Skip this step in REPL fallback (when `$ACTIONS_DIR` is unset).**

Write a compact scan summary to `$ACTIONS_DIR/scan-summary.txt`. A few lines is enough — this is input for slug generation, not a full briefing. Include:
- Titles of the unanswered questions (one per line)
- Names of currently alerting monitors (if Datadog ran)
- A one-line overall theme for the scan

### 6. Generate slug

**Skip this step in REPL fallback.**

Invoke `krust oncall slug --from $ACTIONS_DIR/scan-summary.txt` via the Bash tool. Capture stdout as `<slug>`. Do NOT do inline slug reasoning — always defer to `krust oncall slug`.

### 7. Compute final path

**Skip this step in REPL fallback.**

Compute the final briefing path: `$KRUST_OUT_DIR/$KRUST_OUT_DATE-<slug>.md`. If that path already exists, append `-2`, `-3`, ... until a free path is found. Keep the chosen path in a variable — you'll write to it in Step 10.

### 8. Create monitoring crons

Cron names follow the pattern `oncall-<slug>-<type>`, where `<type>` is `slack` or `datadog`. In REPL fallback, substitute an inline timestamp slug (e.g., `oncall-20260423-1430-slack`) since `<slug>` is not computed.

**Slack questions loop** (every 10 minutes, always created):
```
CronCreate name="oncall-<slug>-slack" cron="*/10 * * * *" recurring=true
```
Prompt: Fetch each channel in `$ONCALL_SLACK_CHANNELS`. Compare against the already-answered list. For new unanswered questions, research with /explore and draft with /ghost-write. Report findings inline. The prompt MUST reference `$ONCALL_SLACK_CHANNELS` — no literal channel names.

**Datadog alerts loop** (every 2 minutes, only if `$monitor_filter` is non-empty):
```
CronCreate name="oncall-<slug>-datadog" cron="*/2 * * * *" recurring=true
```
Prompt: Query `mcp__datadog__get_monitors` with `name=$ONCALL_MONITOR_FILTER`, parse with python to extract Alert/Warn status, filter client-side to monitors tagged with `$ONCALL_MONITOR_TEAM_TAG`, diff against baseline. Report new alerts or recoveries. For new alerts, launch /explore to investigate. The prompt MUST reference `$ONCALL_MONITOR_FILTER` and `$ONCALL_MONITOR_TEAM_TAG` — no literal values.

Capture both returned cron IDs from `CronCreate` output.

### 9. Persist bd metadata

**Skip this step in REPL fallback (when `$KRUST_BEADS_ID` is unset).**

Record which crons were created and where the scan summary lives:

```
bd update $KRUST_BEADS_ID --set-metadata='cron_ids=["oncall-<slug>-slack","oncall-<slug>-datadog"]'
bd update $KRUST_BEADS_ID --set-metadata='scan_summary_path=<actions-dir>/scan-summary.txt'
```

The `cron_ids` array must match what was actually created — omit the datadog entry if the Datadog flow was skipped.

### 10. Write the briefing

**In REPL fallback, present this inline in your response instead of writing a file.**

Write the full briefing to the final path computed in Step 7. Include:
- Unanswered-questions section with thread URLs and drafted responses
- Research findings per question with file paths and line numbers
- Alert baseline (if Datadog ran): `alerting` and `nodata` sets, plus chronic-vs-new classifications
- Cron names for human reference (`oncall-<slug>-slack`, optionally `oncall-<slug>-datadog`)

### 11. Emit artifact

**Skip this step in REPL fallback.**

```
krust artifact oncall <slug> "<final-path>"
```

This hands the briefing back to the krust lifecycle.

## Loop iteration behavior

### Slack check (every 10 min)

1. Fetch each channel in `$ONCALL_SLACK_CHANNELS`
2. Compare messages against the already-answered list (maintained in conversation context)
3. If new unanswered questions found:
   - Launch /explore agent per question
   - Draft response with /ghost-write
   - Add to already-answered list
4. If no new questions: report "No new unanswered questions in [channels]"

### Datadog check (every 2 min)

1. Call `mcp__datadog__get_monitors` with `name=$ONCALL_MONITOR_FILTER` (expect the large-response-saved-to-file behavior every time).
2. Parse the result file with the python snippet from Section 4, filtering to `$ONCALL_MONITOR_TEAM_TAG`.
3. Diff current sets against the **working baseline** carried forward in your previous response (not the stale baseline from the cron prompt, which is frozen at session start):
   - **New alerts**: report, launch /explore to investigate, add to working baseline.
   - **Recovered**: report the recovery, remove from working baseline.
   - **No change**: report "No alert changes."
4. Diff No Data set the same way, but **do not spawn /explore for new No Data** — these are almost always broken metric pipelines, not active incidents. Just note them.
5. Restate the updated `alerting` and `nodata` sets at the bottom of your response so the next iteration can resume from them.

**Investigation shortcut — adjacent monitor IDs**: if your monitors live in contiguous Terraform files, adjacent IDs often fire together — one /explore pass covers the family. When one fires, adjacent IDs in the same family often fire together in the next 1–2 iterations, and a single investigation covering the common root cause is enough. Don't launch a fresh investigation for each sibling alert.

**Cadence reality**: re-fetching the API every iteration is fine in practice (~2-5 seconds per call). Skip the "cached rerun" optimization unless you're hitting rate limits — the freshness is worth more than the saved call.

## Rules

- **Never post Slack messages** — all responses are drafted for the user to copy-paste
- **Never write files outside `$KRUST_OUT_DIR` or `$ACTIONS_DIR`** — the briefing and scan summary are the only writes this skill makes
- **Never run git commands** — no commits, no branches
- **All research must be verified** — cite file paths and line numbers, not speculation
- **Use /ghost-write for all response drafts** — maintains voice consistency
- **Use opus for all explore agents** — no exceptions
- **Track already-answered questions in context** — avoid re-researching
- **Present findings inline in loop iterations** — don't summarize away the evidence
- **Alert baseline is additive** — when alerts recover, remove from baseline; when new alerts fire, add to baseline
- **Defer to `krust oncall slug` for slug generation** — no inline slug reasoning
