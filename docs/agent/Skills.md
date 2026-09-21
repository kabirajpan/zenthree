# Skills

## What is a Skill?

A Skill is a reusable set of instructions and metadata describing how an AI agent should approach a class of software-development tasks. A Skill is passive data: it is not a tool, subagent, model, or execution loop.

All Skills in the initial library are language-agnostic. Skills must not encode a specific programming language, framework, build system, package manager, or project convention. Those details must be discovered dynamically from the workspace and the current context before a Skill is applied.

The initial library contains:

1. `repository-exploration`
2. `task-planning`
3. `implementation`
4. `debugging`
5. `testing`
6. `code-review`
7. `refactoring`
8. `verification`
9. `security-review`
10. `git-workflow`
11. `failure-recovery`
12. `dependency-analysis`
13. `performance-analysis`
14. `documentation`
15. `codebase-migration`

## Where Skills live

Workspace Skills are discovered from:

```text
<workspace>/.zenthree/agent/zene/skills/<skill-name>/SKILL.md
```

Each skill directory must contain one `SKILL.md` file. Future support may add `references/`, `scripts/`, `examples/`, or `assets/`, but those are not executed or interpreted by the current subsystem.

The Skill subsystem is platform-neutral and is intended to work on Linux, macOS, and Windows. Skill paths are resolved with Rust's cross-platform `Path` APIs; do not hardcode `/` or `\` in Skill implementations or documentation examples.

## `SKILL.md` structure

```markdown
---
name: debugging
description: Diagnose and fix software bugs systematically.
version: "1.0.0"
tags:
  - debugging
  - testing
---
# Debugging

Instructions written in Markdown.
```

`name` and `description` are required. `version` and `tags` are optional. The Markdown body is retained as the Skill's `instructions`.

## Discovery and validation

Call `agent::skills::discover(workspace)` to scan `.zenthree/agent/zene/skills/`. Each child directory is checked for `SKILL.md`, parsed, validated, and registered. Invalid Skills are reported in `DiscoveryReport::errors`; valid Skills continue loading.

Missing `.zenthree/agent/zene/skills/` is an empty successful discovery. A directory without `SKILL.md`, malformed YAML, missing required metadata, or a duplicate name is reported without preventing other Skills from loading.

## Registry and loading

`SkillRegistry` manages registered Skills with `register`, `get`, `list`, `contains`, and `unregister`. `SkillLoader::load(path)` loads and validates one `SKILL.md`. The registry does not execute instructions or interact with tools, providers, orchestration, or the agent loop.

## Example

Create `.zenthree/agent/zene/skills/testing/SKILL.md`:

```markdown
---
name: testing
description: Design focused regression tests.
tags:
  - testing
---
# Testing

Reproduce the behavior, add a focused test, and run the smallest relevant suite.
```

## Current limitations and future extensions

The current subsystem only discovers, parses, validates, registers, and loads passive Skill data. It does not select Skills, rank them, activate them automatically, inject instructions into prompts, execute scripts, or change permissions.
