---
name: repository-exploration
description: Understand an unfamiliar repository before making targeted changes.
version: "1.0.0"
tags:
  - exploration
  - repository
  - architecture
---
# Repository Exploration

## Purpose

Use this skill when you need to understand an unfamiliar repository before implementing a change.

## Workflow

1. Identify the repository root, language, package manager, and build system.
2. Inspect the top-level structure and locate application entry points.
3. Find the modules, configuration, dependencies, and tests related to the request.
4. Search for existing implementations and reusable helpers before adding new code.
5. Trace the relevant data flow and control flow across module boundaries.
6. Record assumptions and verify them against the source and tests.
7. Check platform-specific behavior on Linux, macOS, and Windows when the change touches paths, processes, terminals, shells, or filesystem behavior.
8. Avoid unrelated changes and preserve established project conventions.

## Rules

- Prefer targeted reads and searches over dumping whole files.
- Treat existing behavior and tests as constraints.
- Do not introduce a duplicate abstraction when a suitable one already exists.
- Prefer platform-neutral APIs and document unavoidable OS-specific behavior.
- Report uncertainty instead of guessing.
