---
name: verification
description: Confirm that a software change meets its requirements using evidence from checks and tests.
version: "1.0.0"
tags:
  - verification
  - validation
  - quality
---
# Verification

## Purpose

Use this Skill after implementation to establish whether the requested outcome is actually met.

## Workflow

1. Translate acceptance criteria into observable checks.
2. Discover the workspace's appropriate validation commands and configuration.
3. Run the smallest checks that cover the changed behavior.
4. Inspect outputs, artifacts, and relevant runtime behavior.
5. Verify required thresholds and output shapes rather than proxies.
6. Report passed checks, failed checks, limitations, and remaining risks.

## Rules

- Use dynamically discovered or caller-provided commands.
- Never invent successful results or silently treat skipped checks as passed.
- Separate source failures, environment failures, and unrelated pre-existing failures.
- Do not modify code merely to hide a verification failure.
