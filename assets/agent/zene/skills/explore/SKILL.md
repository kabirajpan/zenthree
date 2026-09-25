---
name: explore
description: Understand project layout, manifests, entry points, and architecture using exploration tools.
version: "1.0.0"
tags:
  - architecture
  - exploration
  - navigation
---
# Repository Exploration

## Purpose
Use this skill when orienting in a new or unfamiliar codebase, or when locating where specific functionality or architecture is defined.

## Tools
- `list_directory`: View directory layout, file sizes, child counts, and recursive hierarchy (e.g. `depth: 2`).
- `search`: Universal search across `folders`, `files`, `symbols`, and `content`. Combine scopes with `+` (e.g. `in: "symbols+files"`, `in: "files"`).
- `read_file`: Inspect targeted file content safely using `peek: true`, `symbol: "..."`, line ranges, or batch `paths: [...]`.

## Workflow (Depth-First Search)
1. Call `list_directory(path: ".", depth: 2)` to discover the project layout and top-level directory structure dynamically.
2. **Stage 1 (Root First):** Inspect any root-level manifests or configuration files discovered in the root listing before descending into subfolders.
3. **Stage 2 (Descend Branch A):** Select the primary directory branch and traverse down to its leaves, inspecting its entry points and modules until that branch is understood.
4. **Stage 3 (Descend Branch B):** Only after completing the previous branch, move to the next directory branch in depth-first order.
5. When searching for declarations or logic, use `search(query: "...", in: "symbols")` to get AST definition coordinates without noise from call sites.
6. Read standard files directly with `read_file(path: "...")` (it returns up to 500 lines automatically). Only use `peek: true` for very large files (>300 lines) to view outlines first. Never pass arbitrary line ranges like 1..200.

## Rules
- **Strict Project Boundary:** Never navigate to or explore parent directories ('..') or paths outside the current workspace root ('./'). All exploration must stay strictly within the current workspace project root.
- **Follow DFS (Depth-First Search):** Start at root, then explore one directory branch completely before touching sibling branches. Never jump erratically between different folders.
- Everything is dynamic: NEVER guess or speculate file names, extensions, or directory structures.
- Always discover files first using `list_directory` or `search(in: "files")` before attempting to read them. Never call `read_file` on an unconfirmed path.
- Direct whole-file reading: For normal files, call `read_file(path: "...")` directly without `start_line` or `end_line`. Never pass arbitrary line numbers like 1..200 for files that only have 60-70 lines.
- Prefer `search(in: "symbols")` over raw text matching when looking for definitions.
- Use batch `paths: [...]` to inspect related files in a single roundtrip to save latency.
- Never make speculative edits during exploration.
- Never run shell commands (like `ls`, `cat`, `find`) via `run_terminal` to explore files. Use `search` and `read_file`.
