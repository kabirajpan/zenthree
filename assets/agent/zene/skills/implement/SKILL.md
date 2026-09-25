---
name: implement
description: Execute targeted, minimal, style-preserving code modifications with diff discipline using file tools.
version: "1.0.0"
tags:
  - implementation
  - coding
  - refactoring
---
# Implementation

## Purpose
Use this skill when modifying source code, implementing new features, or fixing logic.

## Tools
- `create_plan`: Formulate and register the implementation plan before making modifications.
- `read_file`: Inspect the target file around the modification site.
- `edit_file`: Make surgical, precise find-and-replace edits.
- `write_file`: Create brand-new files (use edit_file for existing files).
- `get_diagnostics`: Verify compiler diagnostics immediately after edits.

## Workflow
1. Formulate and call `create_plan` with:
   - `title`: Short descriptive title.
   - `overview`: Architectural intent and summary.
   - `files`: Array of `{ path, action, purpose }`.
   - `diffs`: Array of `{ file, before, after }`.
   - `steps`: Sequential execution phases.
   - `tests`: Test suites and verification commands.
2. The implementation plan will be saved directly to `implementation_plan.md` on disk. In the AI Chat panel, an **`[ 📄 Implementation Plan ]`** badge and a **`[ ▶ Proceed ]`** button are displayed. Clicking the badge opens `implementation_plan.md` in the buffer tab with Markdown Preview enabled.
3. **Approval Gate:** DO NOT proceed with code modifications, file writes, or command execution until the user approves (by clicking `[ ▶ Proceed ]` or saying "proceed" / "go").
4. Once approved, call `read_file` with line bounds on the target file to inspect prevailing idioms, formatting, and imports.
5. Prefer `edit_file` for existing files. Provide exact matching target text including whitespace.
6. Only use `write_file` when generating a brand-new file from scratch.
7. Call `update_plan_step` as each step is completed (`in_progress`, `completed`).
8. Immediately after modifying files, verify via `get_diagnostics` or project verification tests.
9. If diagnostics show errors, fix them before moving to another file.

## Rules
- Always establish and show an implementation plan before making code modifications.
- Never modify files before the user confirms the plan.
- Keep diffs small, cohesive, and easy to review.
- Never rewrite entire existing files when only a few lines need modification.
- Preserve existing comments, docstrings, and formatting style unless explicitly requested.
- Ensure all new types and dependencies are imported and exported properly.
