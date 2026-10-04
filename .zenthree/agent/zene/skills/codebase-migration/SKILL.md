---
name: codebase-migration
description: Plan and execute a controlled migration of code, structure, data, or interfaces.
version: "1.0.0"
tags:
  - migration
  - compatibility
  - architecture
---
# Codebase Migration

## Purpose

Use this Skill for coordinated changes that move a codebase from one structure, interface, or system to another.

## Workflow

1. Define the source state, target state, compatibility requirements, and rollback plan.
2. Discover the workspace architecture, conventions, dependencies, and validation process.
3. Inventory affected files, interfaces, data, generated artifacts, and external consumers.
4. Divide the migration into safe, verifiable stages.
5. Preserve compatibility or provide an explicit transition when required.
6. Execute each stage with focused checks and record progress.
7. Remove obsolete paths only after dependents are migrated and verified.
8. Review the final diff, documentation, and rollback readiness.

## Rules

- Do not assume a migration strategy without inspecting the current system.
- Avoid mixing unrelated feature work into a migration.
- Preserve recoverability and do not delete source data without approval.
- Discover language, framework, build, and project conventions dynamically.
