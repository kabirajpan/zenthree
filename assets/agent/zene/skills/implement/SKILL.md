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
- `read_file`: Inspect the target file around the modification site.
- `edit_file`: Make surgical, precise find-and-replace edits.
- `write_file`: Create brand-new files (use edit_file for existing files).
- `get_diagnostics`: Verify compiler diagnostics immediately after edits.

## Workflow
1. Call `read_file` with line bounds on the target file to inspect prevailing idioms, formatting, and imports.
2. Prefer `edit_file` for existing files. Provide exact matching target text including whitespace.
3. Only use `write_file` when generating a brand-new file from scratch.
4. Immediately after modifying files, call `get_diagnostics` to ensure no syntax, borrow-check, or type errors were introduced.
5. If diagnostics show errors, fix them before moving to another file.

## Rules
- Keep diffs small, cohesive, and easy to review.
- Never rewrite entire existing files when only a few lines need modification.
- Preserve existing comments, docstrings, and formatting style unless explicitly requested.
- Ensure all new types and dependencies are imported and exported properly.
