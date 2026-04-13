#!/usr/bin/env bash
# Extract persistent memories from bd prime --export output.

command -v bd >/dev/null 2>&1 || exit 0
bd prime --export 2>/dev/null | sed -n '/^## Persistent Memories/,$p'
