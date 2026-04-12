#!/usr/bin/env python3
"""Find ancestor project MEMORY.md files for the current working directory."""

import json
import os
import sys

try:
    cwd = os.getcwd()
    home = os.path.expanduser("~")
    cockpit = os.environ.get("COCKPIT_DIR", home + "/ai-cockpit")
    tree = json.load(open(cockpit + "/project-tree.json"))

    def slug(path):
        expanded = os.path.expanduser(path)
        return "-" + expanded.lstrip("/").replace("/", "-")

    def find_ancestors(projects, cwd, chain=None):
        if chain is None:
            chain = []
        for p in (projects or []):
            path = os.path.expanduser(p.get("path", ""))
            if cwd.startswith(path) and cwd != path:
                mem = home + "/.claude/projects/" + slug(path) + "/memory/MEMORY.md"
                if os.path.exists(mem):
                    chain.append((p["name"], mem))
                find_ancestors(p.get("children", []), cwd, chain)
        return chain

    for name, mem in find_ancestors(tree.get("projects", []), cwd):
        print(f"{name}\t{mem}")
except Exception:
    pass

sys.exit(0)
