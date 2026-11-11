# Skills Subsystem — ZENE

## What is a Skill?

A Skill is a reusable set of instructions, workflow rules, and tool guidance describing how **ZENE** (the Zenthree AI coding assistant) should approach specific classes of software engineering tasks.

Skills are passive, structured Markdown documents with YAML frontmatter. They do not pollute the prompt with thousands of tokens; instead, they are loaded **on-demand** via the `activate_skill` tool when ZENE needs specific workflow guidance.

---

## The 3-Tier Architecture

Skills are organized into a 3-tier hierarchy where higher scopes override lower scopes:

```text
┌────────────────────────────────────────────────────────────────────────┐
│ 1. Global (Built-in)                                                   │
│    • Ships directly inside the binary from assets/agent/zene/skills/   │
│    • Embedded at compile time via include_str!                         │
│    • 6 core skills: explore, plan, implement, debug, verify, review    │
│    • Available out-of-the-box anywhere with ZERO files required.       │
├────────────────────────────────────────────────────────────────────────┤
│ 2. User Base (~/.zenthree/agent/zene/skills/)                          │
│    • Lives in the user's home directory.                               │
│    • Follows the user across all projects on their machine.            │
│    • Overrides Global skills if the same name is defined.              │
├────────────────────────────────────────────────────────────────────────┤
│ 3. Project Base (<workspace>/.zenthree/agent/zene/skills/)             │
│    • Lives inside a specific project repository.                       │
│    • Checked into git for team or repository-specific workflows.       │
│    • Highest precedence: overrides User and Global skills.             │
└────────────────────────────────────────────────────────────────────────┘
```

---

## The 6 Core Built-in Skills

Located in `assets/agent/zene/skills/`:

| Skill | Primary Tools | Purpose |
|---|---|---|
| **`explore`** | `get_project_structure`, `search`, `read_file` | Orient in an unfamiliar project or locate definitions. |
| **`plan`** | `create_plan`, `update_plan_step` | Formulate phased execution plans with verification gates. |
| **`implement`** | `read_file`, `edit_file`, `write_file`, `get_diagnostics` | Surgical, minimal code edits with diff discipline. |
| **`debug`** | `get_diagnostics`, `search`, `read_file`, `edit_file`, `run_terminal` | Diagnose and resolve errors using concrete evidence. |
| **`verify`** | `run_terminal`, `get_diagnostics` | Dynamic checks, test suite execution, and compiler validation. |
| **`review`** | `git_status`, `git_diff`, `get_diagnostics` | Diff review, security checks, and code cleanliness before done. |

---

## `SKILL.md` File Structure

Custom user or workspace skills follow this exact format:

```markdown
---
name: database-migration
description: Safe practices for schema changes and rollbacks.
version: "1.0.0"
tags:
  - database
  - sql
---
# Database Migration

## Purpose
Use this skill when modifying SQL tables or running database migrations.

## Tools
- `run_terminal`: Run migration scripts and validation checks.
- `read_file`: Inspect migration schema files.

## Workflow
1. Inspect existing table schemas before altering columns.
2. Ensure every migration has a corresponding reversible rollback step.
3. Test migrations against the local test database first.

## Rules
- Never drop columns without a migration grace period.
- Never hardcode production database credentials.
```

---

## On-Demand Activation (`activate_skill`)

Rather than bloating system prompts with every skill's full text:
1. The system prompt contains a compact 6-line summary of available skills.
2. When ZENE begins a phase of work, it invokes the tool:
   ```json
   {
     "name": "activate_skill",
     "arguments": { "name": "debug" }
   }
   ```
3. The tool dynamically injects the full markdown workflow and rules into the active conversation.
