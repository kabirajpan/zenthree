---
name: testing
description: Design and run focused tests that verify requested behavior and prevent regressions.
version: "1.0.0"
tags:
  - testing
  - verification
  - regression
---
# Testing

## Purpose

Use this Skill when adding, selecting, or evaluating tests.

## Workflow

1. Discover the workspace's testing tools and conventions dynamically.
2. Identify the behavior, boundaries, and failure modes that need coverage.
3. Choose the smallest meaningful unit, integration, or end-to-end tests.
4. Cover success, invalid input, error handling, and important edge cases.
5. Run focused tests first, then broaden validation when justified.
6. Review failures as evidence and report exact results.

## Rules

- Do not assume a test runner, language, framework, or command.
- Do not weaken or remove a failing test merely to make a suite pass.
- Keep tests deterministic and isolated where possible.
- Match existing test style and avoid unrelated test churn.
