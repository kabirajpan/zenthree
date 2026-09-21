---
name: refactoring
description: Improve internal structure without changing intended external behavior.
version: "1.0.0"
tags:
  - refactoring
  - design
  - maintainability
---
# Refactoring

## Purpose

Use this Skill for behavior-preserving structural improvements.

## Workflow

1. Define the behavior and interfaces that must remain stable.
2. Discover existing abstractions, duplication, dependencies, and conventions.
3. Identify a small cohesive refactoring boundary.
4. Make the structural change in incremental steps.
5. Run focused regression tests after each meaningful step.
6. Review the diff for accidental behavior or API changes.

## Rules

- Do not refactor without a clear maintenance benefit.
- Preserve behavior unless a behavior change is explicitly requested.
- Prefer existing abstractions over introducing parallel patterns.
- Avoid broad formatting or unrelated cleanup.
