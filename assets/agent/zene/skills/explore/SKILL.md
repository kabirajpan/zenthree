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
- `find_files`: Locate configuration and manifest files by name pattern.
- `search_codebase`: Find symbol definitions, types, or function calls across files.
- `read_file`: Inspect targeted file content using line ranges.

## Workflow
1. Call `get_project_structure` to understand the root layout and top-level modules.
2. Use `find_files` to locate package manifests (`Cargo.toml`, `package.json`, `go.mod`, etc.) to confirm languages and dependencies.
3. Locate entry points (`main.rs`, `lib.rs`, `index.ts`, `app.py`) and inspect them.
4. When searching for symbols, call `search_codebase` rather than scanning files manually.
5. Always call `read_file` with `start_line` and `end_line` parameters to inspect relevant slices without blowing out token limits.

## Rules
- Prefer targeted reads over loading entire large files into memory.
- Identify established state management, error handling, and naming patterns before making changes.
- Never make speculative edits during exploration.
