---
name: verify
description: Validate changes using dynamic checks, tests, compiler diagnostics, and linters to prove correctness.
version: "1.0.0"
tags:
  - verification
  - testing
  - quality-assurance
---
# Verification

## Purpose
Use this skill to prove that code changes work correctly and introduce zero regressions before marking a task complete.

## Tools
- `run_terminal`: Execute targeted test commands, linter runs, or build checks.
- `get_diagnostics`: Confirm that all compiler diagnostics and language-server checks pass with zero errors.

## Workflow
1. **Identify Relevant Checks**: Determine the most targeted test or build command that exercises the modified code according to the project's build system.
2. **Execute Targeted Tests**: Use `run_terminal` to run the specific test suite.
3. **Verify Reported Fix**: Check the terminal output to confirm that the reported bug is resolved.
4. **Run Full Diagnostics**: Call `get_diagnostics` to verify that no new compile errors or warnings were introduced across the workspace.
5. **Check Unrelated Tests**: Ensure that existing, unmodified tests continue to pass with zero regressions.

## Rules
- Always run dynamic automated verification when a build or test suite exists.
- Run the smallest relevant test suite first to get fast feedback.
- If verification fails, treat the failure output as actionable feedback to fix the issue—do not claim success if tests fail.
- Do not add fake passing assertions or skip failing tests.
