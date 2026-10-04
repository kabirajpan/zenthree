---
name: debugging
description: Diagnose and fix software failures systematically using evidence.
version: "1.0.0"
tags:
  - debugging
  - errors
  - testing
---
# Debugging

## Purpose

Use this skill when investigating a software failure or unexpected behavior.

## Workflow

1. Reproduce the failure and capture the exact error.
2. Read the error, stack trace, logs, and relevant environment details.
3. Inspect the smallest relevant code path.
4. Trace execution from the failing input to the observed result.
5. Form a concrete hypothesis about the root cause.
6. Test the hypothesis with focused checks or a regression test.
7. Apply the smallest appropriate fix.
8. Check whether the failure is platform-specific across Linux, macOS, and Windows.
9. Run regression tests and verify the original failure is resolved.

## Rules

- Prefer evidence over assumptions.
- Separate the root cause from unrelated pre-existing issues.
- Do not hide errors with broad catches or silent fallbacks.
- Compare path, shell, process, permissions, and environment behavior across supported platforms.
- Keep fixes narrow and review the final diff.
