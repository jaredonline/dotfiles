#!/usr/bin/env bash
# Print the path to the latest daily summary file, or empty if none exist.

COCKPIT_DIR="${COCKPIT_DIR:-$HOME/ai-cockpit}"
result=$(ls -1 "$COCKPIT_DIR/state/news"/????/??/????-??-??.md 2>/dev/null | tail -1)
[ -n "$result" ] && printf '%s\n' "$result"
exit 0
