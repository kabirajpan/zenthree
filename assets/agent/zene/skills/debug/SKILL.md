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
- `search_codebase`: Locate error strings, panic messages, or failing functions across the codebase.
- `read_file`: Inspect the failing code path with precise line ranges.
- `edit_file`: Apply the surgical root-cause fix.
- `run_terminal`: Run test suites or commands to reproduce and confirm fixes.

## Workflow
1. **Capture & Reproduce**: Read the exact error message, backtrace, or test failure output. If necessary, use `run_terminal` to reproduce the failure.
2. **Inspect Diagnostics**: Call `get_diagnostics` to see if the compiler or language server is already reporting issues.
3. **Trace Execution**: Use `search_codebase` and `read_file` to trace the code path from the failing input to the point of failure.
4. **Hypothesis Formulation**: Formulate a testable hypothesis explaining *why* the failure occurs before making edits.
5. **Apply Minimal Fix**: Use `edit_file` to apply the narrowest fix that addresses the root cause directly.
6. **Verify Resolution**: Re-run the reproducing test or `get_diagnostics` to confirm the error is eliminated and no regressions occurred.

## Rules
- Prefer concrete compiler/log evidence over assumptions.
- Never mask failures using broad catch-alls, empty error branches, or silent fallbacks.
- Fix the actual root cause rather than patching superficial symptoms.
- Always check that surrounding code and callers remain intact.
