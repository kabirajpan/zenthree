---
name: dependency-analysis
description: Understand and safely evaluate project dependencies and their impact.
version: "1.0.0"
tags:
  - dependencies
  - compatibility
  - supply-chain
---
# Dependency Analysis

## Purpose

Use this Skill when adding, upgrading, removing, or investigating dependencies.

## Workflow

1. Discover the package manager, manifests, lockfiles, workspace layout, and platform constraints.
2. Trace direct and transitive usage relevant to the requested change.
3. Evaluate compatibility, licensing, security, size, performance, and maintenance impact.
4. Check lockfile and reproducibility implications.
5. Make the smallest manifest change required.
6. Run the workspace's dependency, build, and test validation.

## Rules

- Do not assume a package manager or dependency layout.
- Prefer existing dependencies when they satisfy the requirement.
- Do not update unrelated dependencies opportunistically.
- Never commit secrets or unverified dependency artifacts.
