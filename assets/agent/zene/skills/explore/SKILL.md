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
- `get_project_structure`: View directory layout and folder hierarchy.
- `search`: Universal search across `folders`, `files`, `symbols`, and `content`. Combine scopes with `+` (e.g. `in: "symbols+files"`, `in: "files"`).
- `read_file`: Inspect targeted file content safely using `peek: true`, `symbol: "..."`, line ranges, or batch `paths: [...]`.

## Workflow
1. Call `get_project_structure` to understand the root layout and top-level modules.
2. Use `search(query: "toml,json,mod", in: "files")` to locate package manifests (`Cargo.toml`, `package.json`, `go.mod`) and confirm languages/dependencies.
3. Locate entry points (`main.rs`, `lib.rs`, `index.ts`, `app.py`) and inspect them.
4. When searching for where things are defined, use `search(query: "...", in: "symbols")` to get AST definition coordinates without noise from call sites.
5. Inspect large or unfamiliar files with `read_file(path: "...", peek: true)` first to see line count and outline without burning tokens.
6. Read functions directly with `read_file(path: "...", symbol: "fn_name")` or inspect related files in one roundtrip via `read_file(paths: ["a.rs", "b.rs"])`.

## Rules
- Always check file line counts before reading: files >500 lines should be peeked or read by symbol/range.
- Prefer `search(in: "symbols")` over raw text matching when looking for definitions.
- Use batch `paths: [...]` to inspect related types/modules in a single roundtrip to save latency.
- Never make speculative edits during exploration.
