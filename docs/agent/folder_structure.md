# Agent Crate Refactor — Folder Structure Plan

## What you have vs. what the flow diagram requires

---

## Current Structure (what exists)

```
crates/agent/src/
├── lib.rs                         ← create_agent(), create_agent_for_model()
├── agentic_loop/                  ← [COMPLETED] Core multi-turn autonomous execution
│   ├── mod.rs
│   └── engine.rs                  ← Agent struct, streaming event loop, cancellation checkpoints
├── context/                       ← [COMPLETED] Context window pruning & token guardrails
│   ├── mod.rs
│   └── pruner.rs                  ← History pruning, sanitize_tool_result, tool compaction & 413 auto-prune
├── orchestrator/                  ← [COMPLETED] Orchestration configuration & sessions
│   ├── mod.rs
│   ├── config.rs                  ← AgentConfig (max_iterations, token budgets, retries, verification)
│   └── session.rs                 ← AgentSession (JSON conversation export/import)
├── verification/                  ← [COMPLETED] Dynamic post-loop compiler / lint verification
│   ├── mod.rs
│   └── runner.rs                  ← VerificationRunner (100% dynamic: NO static whitelists or hardcoded commands; feeds errors back to loop)
├── provider/                      ← [COMPLETED] LLM Provider Streaming Adapters
│   ├── mod.rs                     ← create_default_provider(), create_provider_for()
│   ├── gemini.rs                  ← GeminiProvider (streaming, tool schema)
│   └── groq.rs                    ← GroqProvider (streaming, tool schema)
├── tools/                         ← [COMPLETED] 15 Built-in IDE Tools
│   ├── mod.rs
│   ├── registry.rs                ← ToolRegistry (minimal / essentials / all)
│   ├── traits.rs
│   ├── diagnostics.rs             ← get_diagnostics (LSP & compiler errors)
│   ├── fs/
│   │   ├── read.rs, write.rs, edit.rs
│   │   ├── delete.rs (ops.rs), rename.rs (ops.rs)
│   │   ├── list.rs, structure.rs
│   │   └── mod.rs
│   ├── git/
│   │   ├── vcs.rs                 ← git_status, git_diff
│   │   └── mod.rs
│   ├── search/
│   │   ├── codebase.rs, files.rs
│   │   └── mod.rs
│   ├── terminal/
│   │   ├── exec.rs                ← run_terminal (dynamic command execution)
│   │   └── mod.rs
│   └── plan/
│       ├── planner.rs             ← create_plan, update_plan_step
│       └── mod.rs
├── traits/                        ← [COMPLETED] Core contracts
│   ├── mod.rs
│   ├── provider.rs                ← Provider trait
│   └── tool.rs                    ← Tool trait + ToolRisk enum
└── types/                         ← [COMPLETED] Type definitions
    ├── mod.rs
    ├── approval.rs                ← ApprovalRequest, ApprovalDecision
    ├── error.rs                   ← ProviderError, ToolError
    ├── event.rs                   ← AgentEvent (with tool args for trajectory), ProviderEvent
    └── message.rs                 ← Message, Role, ToolCall
```

---

## Architectural Principles & Guarantees

> [!IMPORTANT]
> 1. **No Static Command Whitelists / No Hardcoded Predefined Commands**:
>    - Verification (`VerificationRunner`) and terminal execution are **100% dynamic**.
>    - We **never** hardcode static whitelists (e.g. hardcoding only `cargo test` or `npm build`).
>    - The caller, workspace configuration, or user dynamically supplies the verification command (`config.verification_command`), allowing the agent to verify Rust, TypeScript, Go, Python, or any other build system without arbitrary restrictions.
> 2. **Targeted Reads over Whole-File Dumps**:
>    - Tools favor 1-indexed slice reads (`start_line`/`end_line`) to avoid blowing token context.
> 3. **Proactive Compaction & 413 Auto-Recovery**:
>    - The agent automatically compresses older tool output and self-heals if payload rate limits are reached.
> 4. **Zero UI Lag**:
>    - Agent background threads communicate via asynchronous MPSC channels; the UI thread never busy-spins or blocks.

---

## Gap Analysis — Status & Progress

| Flow Node | Status | Progress / Details |
|---|---|---|
| **Request** | ✅ COMPLETED | Entry via `lib.rs` (`create_agent`, `create_agent_for_model`, `create_agent_for_model_and_workspace`). |
| **Context Engine** | ⚠️ IN PROGRESS | `context/pruner.rs` completed (prune, sanitize, historical compaction, 413 auto-prune, user query preservation). Next: `workspace.rs`, `git_context.rs`. |
| **Orchestrator** | ✅ COMPLETED | `orchestrator/` + `agentic_loop/engine.rs` completed with non-blocking stream and cancellation tokens. |
| **Skills** | ✅ COMPLETED | 3-tier language-agnostic skill engine (`Global` built-in, `User` home, `Project` workspace). 6 core skills in `assets/agent/zene/skills/` embedded into binary via `include_str!`, plus `activate_skill` tool and IDE chat runtime wiring. |
| **Model (API call)** | ✅ COMPLETED | Gemini & Groq streaming with tool schemas. |
| **Tools** | ✅ COMPLETED | 16 tools across `fs`, `git`, `search`, `terminal`, `plan`, `diagnostics`, `activate_skill`. |
| **Permission gate** | ✅ COMPLETED | `ToolRisk` + `ApprovalRequest` / `ApprovalDecision`. |
| **Subagents** | ❌ PENDING | No mini-loop spawning for isolated sub-tasks yet. |
| **Sandbox** | ❌ PENDING | No isolated execution environment for destructive commands. |
| **Verification** | ✅ COMPLETED | `verification/runner.rs` dynamically runs compiler/test checks with error loop retry. **Strictly NO static whitelists or predefined command restrictions.** |
| **Memory** | ⚠️ PARTIAL | Session JSON persisted; token compaction added; cross-session summary pending. |
| **Evaluation** | ❌ PENDING | No offline task replay harness yet. |
| **Trajectory UI** | ✅ COMPLETED | Cursor-style collapsible exploration groups, file edit diff badges, and sibling markdown answer in IDE. |

---

## Target Structure (Roadmap)

```
crates/agent/src/
├── lib.rs                         ← [COMPLETED]
│
├── agentic_loop/                  ← [COMPLETED]
│   ├── mod.rs
│   └── engine.rs                  ← core run() loop with cancellation, 413 retry & workspace skills
│
├── context/                       ← [IN PROGRESS]
│   ├── mod.rs
│   ├── pruner.rs                  ← [COMPLETED] history pruning, sanitize_tool_result, compaction, 413 auto-prune, user query preservation
│   ├── workspace.rs               ← [PENDING - NEXT] open files, active cursor slice, project directory injection
│   └── git_context.rs             ← [PENDING - NEXT] git status/diff injected into system prompt
│
├── orchestrator/                  ← [COMPLETED]
│   ├── mod.rs
│   ├── config.rs                  ← (iterations, token budgets, retries, verification)
│   ├── planner.rs                 ← [PENDING] picks skills/strategy before loop runs
│   └── session.rs                 ← conversation export/import JSON
│
├── verification/                  ← [COMPLETED]
│   ├── mod.rs
│   └── runner.rs                  ← VerificationRunner: runs build/test, feeds failure back to model
│
├── skills/                        ← [COMPLETED]
│   ├── mod.rs
│   ├── skill.rs                   ← Skill, SkillMetadata, SkillScope (Global, User, Project)
│   ├── builtin.rs                 ← 6 core skills embedded directly into binary from assets/
│   ├── discovery.rs               ← 3-tier discovery (Builtin, ~/.zenthree, <workspace>/.zenthree)
│   ├── loader.rs                  ← SkillLoader::load with scope
│   ├── parser.rs                  ← YAML frontmatter parser with CRLF normalization
│   └── registry.rs                ← In-memory registry with scope precedence & prompt_summary
│
├── subagent/                      ← [PENDING]
│   ├── mod.rs
│   └── runner.rs                  ← SpawnedAgent: runs isolated sub-tasks, returns summary to parent
│
├── memory/                        ← [PENDING]
│   ├── mod.rs
│   └── summarizer.rs              ← compresses old multi-session conversations into executive summary
│
├── evaluation/                    ← [PENDING] offline task replay harness
│   ├── mod.rs
│   ├── harness.rs
│   └── tasks/                     ← .json task definitions
│
├── provider/                      ← [COMPLETED]
│   ├── mod.rs
│   ├── gemini.rs
│   └── groq.rs
│
├── tools/                         ← [COMPLETED]
│   └── fs/, git/, search/, terminal/, plan/, diagnostics/, skill.rs (activate_skill), registry.rs
│
├── traits/                        ← [COMPLETED]
│   ├── mod.rs
│   ├── provider.rs
│   └── tool.rs
│
└── types/                         ← [COMPLETED]
    ├── mod.rs
    ├── approval.rs
    ├── error.rs
    ├── event.rs
    └── message.rs
```

---

## Summary of Completed vs. Pending Items

| Action | Item | Status |
|---|---|---|
| **NEW dir** | `agentic_loop/` — `engine.rs` | ✅ COMPLETED |
| **RENAME** | `orchestrator/context.rs` → `context/pruner.rs` | ✅ COMPLETED |
| **RENAME** | `orchestrator/state.rs` → `orchestrator/session.rs` | ✅ COMPLETED |
| **NEW dir** | `verification/` — `runner.rs` (100% dynamic, NO static whitelists or predefined commands) | ✅ COMPLETED |
| **TOKEN GUARDRAILS** | Compaction & 413 auto-prune in `context/pruner.rs` & `agentic_loop/engine.rs` | ✅ COMPLETED |
| **TRAJECTORY UI** | Cursor/Antigravity style collapsible groups & diff badges in `zenthree` | ✅ COMPLETED |
| **NEW dir** | `skills/` — 3-tier language-agnostic skill engine (`Global`, `User`, `Project`) + 6 embedded core skills (`assets/agent/zene/skills/`) + `activate_skill` tool | ✅ COMPLETED |
| **NEW files** | `context/workspace.rs`, `context/git_context.rs` | ⏳ NEXT IMMEDIATE |
| **NEW dir** | `subagent/` — mini-loop runner for isolated tasks | ⏳ PENDING |
| **NEW dir** | `memory/` — cross-session conversation summarizer | ⏳ PENDING |
| **NEW dir** | `evaluation/` — offline task replay harness | ⏳ PENDING |
| **KEEP** | `provider/`, `tools/`, `traits/`, `types/` | ✅ COMPLETED |

> [!IMPORTANT]
> Milestone Progress:
> 1. **verification/** — ✅ COMPLETED (closes the fail→retry loop)
> 2. **skills/** — ✅ COMPLETED (3-tier engine + embedded core skills + on-demand activation)
> 3. **context/workspace.rs** — ⏳ NEXT IMMEDIATE (injects open file tabs + active cursor slice into prompt)
>
> `subagent/`, `memory/`, and `evaluation/` will follow in subsequent phases.
