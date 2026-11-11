---
name: plan
description: Formulate structured, phased execution plans and track live progress using planning tools.
version: "1.0.0"
tags:
  - planning
  - architecture
  - task-management
---
# Task Planning

## Purpose
Use this skill when approaching complex, multi-phase tasks, refactoring, or tasks with architectural implications.

## Tools
- `create_plan`: Initialize a structured, phased plan with discrete steps.
- `update_plan_step`: Update the status (`in_progress`, `completed`, `failed`) of a plan step.

## Workflow
1. Identify all crates, modules, interfaces, and dependencies impacted by the request.
2. Formulate a plan broken into logical, dependency-ordered phases.
3. Call `create_plan` with the list of steps so the user can track progress in the UI.
4. Each phase must define a specific verification gate (compiler check, unit test, or diagnostic).
5. As each phase begins, call `update_plan_step` with `in_progress`.
6. Once verified, call `update_plan_step` with `completed`.

## Rules
- Keep plans focused on minimal necessary modifications.
- Do not modify source code until the plan steps are initialized.
- If a phase encounters unexpected blockers, update the step status and re-evaluate before proceeding.
