---
name: debug
description: Diagnose and resolve software failures systematically using concrete evidence, diagnostics, and root-cause analysis.
version: "1.0.0"
tags:
  - debugging
  - troubleshooting
  - error-recovery
---
# Debugging

## Purpose
Use this skill when investigating a runtime crash, compiler error, broken test, or unexpected API response.

## Tools
- `get_diagnostics`: Inspect active compiler diagnostics, syntax errors, and type errors.
- `search`: Universal search for error strings, panic messages, or function/struct definitions using scopes like `in: "symbols+content"` or `in: "symbols"`.
- `read_file`: Inspect failing code paths with precise line ranges, AST symbol extraction (`symbol: "..."`), or structure overview (`peek: true`).
- `edit_file`: Apply the surgical root-cause fix.
- `run_terminal`: Run test suites or commands to reproduce and confirm fixes.

## Workflow
1. **Capture & Reproduce**: Read the exact error message, backtrace, or test failure output. If necessary, use `run_terminal` to reproduce the failure.
2. **Inspect Diagnostics**: Call `get_diagnostics` to see if the compiler or language server is already reporting issues.
3. **Trace Execution**: Use `search(query: "...", in: "symbols+content")` to locate error strings, call sites, and definitions. Inspect key definitions or failing regions with `read_file(path: "...", symbol: "...")` or line-bounded reads.
4. **Hypothesis Formulation**: Formulate a testable hypothesis explaining *why* the failure occurs before making edits.
5. **Apply Minimal Fix**: Use `edit_file` to apply the narrowest fix that addresses the root cause directly.
6. **Verify Resolution**: Re-run the reproducing test or `get_diagnostics` to confirm the error is eliminated and no regressions occurred.

## Rules
- Prefer concrete compiler/log evidence over assumptions.
- Never mask failures using broad catch-alls, empty error branches, or silent fallbacks.
- Fix the actual root cause rather than patching superficial symptoms.
- Always check that surrounding code and callers remain intact.
