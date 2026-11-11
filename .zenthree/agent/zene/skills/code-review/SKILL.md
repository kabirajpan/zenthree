---
name: code-review
description: Evaluate a change for correctness, regressions, maintainability, and scope.
version: "1.0.0"
tags:
  - review
  - quality
  - correctness
---
# Code Review

## Purpose

Use this Skill to review a proposed or completed change.

## Workflow

1. Understand the requested behavior and review scope.
2. Inspect the relevant diff and surrounding code.
3. Check correctness, error handling, security, compatibility, and user-visible behavior.
4. Check tests and identify missing coverage.
5. Distinguish actionable defects from suggestions and pre-existing issues.
6. Report findings with severity, evidence, and precise file locations.

## Rules

- Review the actual diff rather than assumptions about intent.
- Prioritize concrete defects and regressions.
- Do not request unrelated refactors.
- Do not claim validation that was not performed.
