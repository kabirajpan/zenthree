---
name: failure-recovery
description: Recover safely from failed commands, partial changes, and interrupted workflows.
version: "1.0.0"
tags:
  - recovery
  - troubleshooting
  - resilience
---
# Failure Recovery

## Purpose

Use this Skill when a workflow fails or leaves an uncertain partial state.

## Workflow

1. Preserve the exact error, current status, and available artifacts.
2. Determine what completed, what failed, and what may have been partially changed.
3. Identify the smallest safe next diagnostic or recovery step.
4. Protect uncommitted work and avoid compounding the failure.
5. Retry only after addressing the observed cause or isolating the environment issue.
6. Verify repository, build, runtime, and data state after recovery.

## Rules

- Do not conceal failures with success-shaped fallbacks.
- Do not repeat destructive operations blindly.
- Distinguish code defects from environment, policy, dependency, and tooling failures.
- Ask for confirmation before irreversible recovery actions.
