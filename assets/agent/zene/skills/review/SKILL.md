---
name: review
description: Inspect diffs for correctness, security vulnerabilities, edge cases, and maintainability before task completion.
version: "1.0.0"
tags:
  - review
  - security
  - code-quality
---
# Code Review

## Purpose
Use this skill to inspect proposed changes, check for regressions, ensure security, and review diff cleanliness before completing a task.

## Tools
- `git_status`: Inspect modified, staged, and untracked files in the repository.
- `git_diff`: Review the exact line-by-line diff of all changes.
- `get_diagnostics`: Confirm zero remaining errors or lint issues.

## Workflow
1. **Check Git Status**: Call `git_status` to see which files were modified, created, or deleted. Verify that only files relevant to the task were touched.
2. **Review Line Diff**: Call `git_diff` to review every single addition and deletion.
3. **Security Audit**: Inspect the diff to guarantee that:
   - No API keys, passwords, bearer tokens, or sensitive env values are hardcoded.
   - User inputs are properly validated and sanitized.
4. **Code Cleanliness**: Ensure that no temporary debug print statements, scratch scripts, or commented-out blocks remain.
5. **Final Diagnostics Check**: Call `get_diagnostics` to verify a clean compile state before declaring completion.

## Rules
- Never commit or stage credentials or sensitive information.
- Reject unnecessary refactoring or style changes outside the requested task scope.
- Confirm all new and existing tests pass before concluding the review.
