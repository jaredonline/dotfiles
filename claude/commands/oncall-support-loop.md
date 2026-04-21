You are running an on-call support loop. Your goal is to monitor Slack channels for unanswered questions and Datadog for alert changes, research issues, and draft responses for the on-call engineer.

## Input

The user optionally provides via ARGUMENTS:
- One or more Slack channels to monitor for questions (e.g., `#your-channel`)
- A Datadog monitor name filter for alert monitoring (e.g., `--monitors=statsig`)

**Defaults** (used when no arguments are provided):
- `slack_channels`: `#your-channel`
- `monitor_filter`: `statsig` (name filter for Datadog API query)
- `monitor_team_tag`: `team:your-team` (post-query filter to scope to the team's monitors)

Parse ARGUMENTS to extract:
- `slack_channels`: all tokens starting with `#` (default: `#your-channel`)
- `monitor_filter`: value after `--monitors=` (default: `statsig`)
- `monitor_team_tag`: value after `--team=` (default: `team:your-team`)

Note: The Datadog `get_monitors` API only supports name-based filtering. Tag-based filtering times out. So we query by name, then filter results client-side to monitors tagged with `monitor_team_tag`.

**Tag casing caveat**: some monitors are double-tagged with both kebab-case and underscore forms (`team:your-team` AND `team:your_team`). Checking one is enough; the kebab-case is the canonical form.

## Agent Strategy

| Step | Parallel? | Why |
|---|---|---|
| 1. Initial scan | No | Establish baseline of existing questions and alert states |
| 2. Research open questions | Yes | One /explore agent per unanswered question |
| 3. Ghost-write responses | No | Depends on research results |
| 4. Baseline alerts | No | Query Datadog monitors to establish known alert set |
| 5. Start loop | No | Set up cron jobs for ongoing monitoring |

## Process

### 1. Initial Slack scan

For each Slack channel:
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
> I'm on call for [team]. Someone asked in Slack:
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

### 4. Baseline Datadog alerts (if --monitors provided)

If a monitor filter was provided:

1. Call `mcp__datadog__get_monitors` with `name=<monitor_filter>`. The response is always too large (~615K chars for a typical team-scoped query) and will be saved to a file rather than returned inline. That's expected — do not retry expecting a smaller response.
2. Parse the saved file with python. The wrapper is `[{"type": "text", "text": "..."}]` and the `text` field has a `Monitors: ` prefix before the JSON array, which must be stripped:
   ```python
   import json, re
   with open(PATH) as f: wrapper = json.load(f)
   text = wrapper[0]["text"]
   m = re.search(r'Monitors:\s*(\[.*\])\s*$', text, re.DOTALL)
   monitors = json.loads(m.group(1))
   team = [x for x in monitors if "team:your-team" in x.get("tags", [])]
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

### 5. Start monitoring loops

Set up two cron jobs:

**Slack questions loop** (every 10 minutes):
```
CronCreate with cron="*/10 * * * *", recurring=true
```
Prompt: Fetch each monitored Slack channel. Compare against the already-answered list. For new unanswered questions, research with /explore and draft with /ghost-write. Report findings inline.

**Datadog alerts loop** (every 2 minutes, only if --monitors provided):
```
CronCreate with cron="*/2 * * * *", recurring=true
```
Prompt: Query monitors, parse with python to extract Alert/Warn status, diff against baseline. Report new alerts or recoveries. For new alerts, launch /explore to investigate.

Tell the user:
```
Monitoring active:
- Slack questions: [channels] (every 10 min, job ID: <id>)
- Datadog alerts: [monitor filter] (every 2 min, job ID: <id>)
- Cancel with: CronDelete <id>
- Auto-expires after 7 days
```

## Loop iteration behavior

### Slack check (every 10 min)

1. Fetch each channel
2. Compare messages against the already-answered list (maintained in conversation context)
3. If new unanswered questions found:
   - Launch /explore agent per question
   - Draft response with /ghost-write
   - Add to already-answered list
4. If no new questions: report "No new unanswered questions in [channels]"

### Datadog check (every 2 min)

1. Call `mcp__datadog__get_monitors` with the name filter (expect the large-response-saved-to-file behavior every time).
2. Parse result file with the python snippet from Section 4 to extract Alert/Warn and No Data monitors.
3. Diff current sets against the **working baseline** carried forward in your previous response (not the stale baseline from the cron prompt, which is frozen at session start):
   - **New alerts**: report, launch /explore to investigate, add to working baseline.
   - **Recovered**: report the recovery, remove from working baseline.
   - **No change**: report "No alert changes."
4. Diff No Data set the same way, but **do not spawn /explore for new No Data** — these are almost always broken metric pipelines, not active incidents. Just note them.
5. Restate the updated `alerting` and `nodata` sets at the bottom of your response so the next iteration can resume from them.

**Investigation shortcut — adjacent monitor IDs**: monitors defined in the same Terraform file get contiguous IDs (e.g., `10000000`–`10000009` all live in `monitors/your-team/`). When one fires, adjacent IDs in the same family often fire together in the next 1–2 iterations. One /explore pass covering the family's common root cause is enough — don't launch a fresh investigation for each sibling alert.

**Cadence reality**: re-fetching the API every iteration is fine in practice (~2-5 seconds per call). Skip the "cached rerun" optimization unless you're hitting rate limits — the freshness is worth more than the saved call.

## Rules

- **Never post Slack messages** -- all responses are drafted for the user to copy-paste
- **Never write files** -- this is a read-only monitoring skill
- **Never run git commands** -- no commits, no branches
- **All research must be verified** -- cite file paths and line numbers, not speculation
- **Use /ghost-write for all response drafts** -- maintains voice consistency
- **Use opus for all explore agents** -- no exceptions
- **Track already-answered questions in context** -- avoid re-researching
- **Present findings inline** -- don't summarize away the evidence
- **Alert baseline is additive** -- when alerts recover, remove from baseline; when new alerts fire, add to baseline
- **No beads task tracking** -- this is a utility/monitoring skill, not tracked work
