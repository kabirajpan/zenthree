---
name: git-workflow
description: Manage version-control changes safely and transparently.
version: "1.0.0"
tags:
  - git
  - branches
  - collaboration
---
# Git Workflow

## Purpose

Use this Skill for branch, diff, commit, synchronization, and collaboration tasks.

## Workflow

1. Inspect repository identity, current branch, status, remotes, and history.
2. Identify the intended scope and protect unrelated local changes.
3. Fetch before comparing remote state when synchronization is needed.
4. Review divergence and conflicts before merge, rebase, or reset operations.
5. Stage only intended files and review the staged diff.
6. Use clear commit messages and verify the resulting history.
7. Push or alter shared history only with explicit authorization.

## Rules

- Never discard local work without explicit approval.
- Never push without explicit user consent.
- Do not use destructive history or filesystem operations as shortcuts.
- Treat nested repositories and generated files as separate scope requiring inspection.
