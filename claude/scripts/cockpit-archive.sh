#!/usr/bin/env bash
# Archive a cockpit state file to finished/ and commit+push.
# Usage: cockpit-archive.sh <state-subdir> <filename> <commit-message>
# Example: cockpit-archive.sh designs design-foo.md "finished: foo"

COCKPIT_DIR="${COCKPIT_DIR:-$HOME/ai-cockpit}"
subdir="$1"
filename="$2"
msg="$3"
src="$COCKPIT_DIR/state/$subdir/$filename"

if [ ! -f "$src" ]; then
  echo "Warning: $src not found, skipping" >&2
  exit 0
fi

mkdir -p "$COCKPIT_DIR/state/$subdir/finished"
mv "$src" "$COCKPIT_DIR/state/$subdir/finished/$filename"
echo "Archived $filename"

git -C "$COCKPIT_DIR" add -A state/ 2>/dev/null
git -C "$COCKPIT_DIR" commit -m "$msg" 2>/dev/null || echo "Warning: git commit failed" >&2
git -C "$COCKPIT_DIR" push 2>/dev/null || echo "Warning: git push failed" >&2

exit 0
