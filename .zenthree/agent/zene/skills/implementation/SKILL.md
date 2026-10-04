---
name: implementation
description: Implement a requested software change with focused, verifiable edits.
version: "1.0.0"
tags:
  - implementation
  - coding
  - testing
---
# Implementation

## Purpose

Use this skill when implementing a requested software change in an existing codebase.

## Workflow

1. Understand the requirement and define the expected behavior.
2. Inspect the current implementation, call sites, and tests.
3. Identify the smallest set of affected files.
4. Plan the change around existing patterns, types, and error handling.
5. Make precise changes and avoid unrelated cleanup.
6. Add or update focused tests for changed behavior.
7. Preserve support for Linux, macOS, and Windows; use platform-neutral APIs where possible.
8. Run the smallest relevant formatter, build, and test commands.
9. Review the resulting diff for scope, correctness, and accidental changes.

## Architectural Guarantees

- Keep verification commands dynamic: use the caller, workspace configuration, or user-provided command instead of creating a static whitelist or hardcoding tool-specific commands.
- Prefer targeted, 1-indexed file reads or slices when the project provides them; avoid dumping whole files when a focused range is sufficient.
- Preserve context guardrails. Compact or truncate oversized historical and tool output through existing helpers instead of bypassing token or payload limits.
- Keep UI-responsive work non-blocking. Use the project's existing background-worker and channel patterns for network, tool, or verification operations; never busy-spin or block the UI thread.

## Rules

- Preserve existing behavior outside the requested change.
- Prefer reusable existing helpers over duplicate logic.
- Surface errors clearly; do not silently ignore failures.
- Keep public API changes intentional.
- Do not assume a Unix shell, Windows path syntax, or a single platform-specific tool unless the requirement explicitly calls for it.
- Do not add static command restrictions, predefined verification lists, or hidden execution fallbacks.
- Do not replace targeted context handling with whole-file dumps.
- Do not introduce synchronous long-running work into an interactive UI path.
