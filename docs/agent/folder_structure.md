# Agent Crate Refactor — Folder Structure Plan

## What you have vs. what the flow diagram requires

---

## Current Structure (what exists)

```
crates/agent/src/
├── lib.rs                         ← create_agent(), create_agent_for_model()
├── orchestrator/
│   ├── mod.rs
│   ├── config.rs                  ← AgentConfig (max_iterations, retry limits, etc.)
│   ├── context.rs                 ← ContextManager (prune_history, sanitize_tool_result)
│   ├── loop_engine.rs             ← Agent struct + full agentic loop (run, tool dispatch, approval)
│   └── state.rs                   ← AgentSession (conversation persistence, export/import)
├── provider/
│   ├── mod.rs                     ← create_default_provider(), create_provider_for()
│   ├── gemini.rs                  ← GeminiProvider (streaming, tool schema)
│   └── groq.rs                    ← GroqProvider (streaming, tool schema)
├── tools/
│   ├── mod.rs
│   ├── registry.rs                ← ToolRegistry (minimal / essentials / all)
│   ├── traits.rs
│   ├── diagnostics.rs             ← get_diagnostics
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
│   │   ├── exec.rs                ← run_terminal
│   │   └── mod.rs
│   └── plan/
│       ├── planner.rs             ← create_plan, update_plan_step
│       └── mod.rs
├── traits/
│   ├── mod.rs
│   ├── provider.rs                ← Provider trait
│   └── tool.rs                    ← Tool trait + ToolRisk enum
└── types/
    ├── mod.rs
    ├── approval.rs                ← ApprovalRequest, ApprovalDecision
    ├── error.rs                   ← ProviderError, ToolError
    ├── event.rs                   ← AgentEvent, ProviderEvent
    └── message.rs                 ← Message, Role, ToolCall
```

---

## Gap Analysis — What's Missing

| Flow Node | Status | Missing |
|---|---|---|
| **Request** | ✅ exists (entry via `lib.rs`) | — |
| **Context Engine** | ⚠️ partial | Only prune/sanitize. No code symbol extraction, no git diff injection, no open-file context |
| **Orchestrator** | ✅ exists (`loop_engine.rs`) | Planning step (picks skills/strategy before loop starts) |
| **Skills** | ❌ missing | No skill system — the loop has no per-task instruction sets |
| **Model (API call)** | ✅ exists | — |
| **Tools** | ✅ exists | — |
| **Permission gate** | ✅ exists (ToolRisk + approval) | — |
| **Subagents** | ❌ missing | No mini-loop spawning for isolated sub-tasks |
| **Sandbox** | ❌ missing | No isolated execution environment for destructive commands |
| **Verification** | ❌ missing | No post-loop build/lint/test check with retry-back-to-loop |
| **Memory** | ⚠️ partial | Session JSON persisted but no summary compression across sessions |
| **Evaluation** | ❌ missing | No offline task replay harness |

---

## Proposed Target Structure

```
crates/agent/src/
├── lib.rs
│
├── context/                       ← [RENAME from orchestrator/context.rs]
│   ├── mod.rs
│   ├── pruner.rs                  ← history pruning, sanitize_tool_result
│   ├── workspace.rs               ← [NEW] open files, symbols, current dir injection
│   └── git_context.rs             ← [NEW] git status/diff injected into system prompt
│
├── orchestrator/
│   ├── mod.rs
│   ├── config.rs                  ← (keep)
│   ├── planner.rs                 ← [NEW] picks skills/strategy before loop runs
│   └── session.rs                 ← [RENAME from state.rs] conversation export/import
│
├── agentic_loop/                  ← [NEW dir — extracted from orchestrator/loop_engine.rs]
│   ├── mod.rs
│   ├── engine.rs                  ← core run() loop (moved from loop_engine.rs)
│   ├── tool_dispatch.rs           ← [NEW split] tool call execution + error feedback
│   ├── approval.rs                ← [NEW split] permission gate logic (ToolRisk checks)
│   └── cancellation.rs           ← [NEW split] cancellation token handling
│
├── skills/                        ← [NEW]
│   ├── mod.rs
│   ├── skill.rs                   ← Skill trait (name, system_prompt, tools_override)
│   ├── coding.rs                  ← "write/edit code" skill
│   ├── debugging.rs               ← "diagnose and fix" skill
│   └── explain.rs                 ← "explain codebase" skill
│
├── subagent/                      ← [NEW]
│   ├── mod.rs
│   └── runner.rs                  ← SpawnedAgent: runs a sub-task, returns result to parent
│
├── verification/                  ← [NEW]
│   ├── mod.rs
│   ├── runner.rs                  ← runs build/lint/test → Pass | Fail(reason) → retry loop
│   └── strategies.rs              ← cargo build, cargo test, eslint, etc.
│
├── memory/                        ← [NEW]
│   ├── mod.rs
│   └── summarizer.rs              ← compresses old conversations into a short summary
│
├── evaluation/                    ← [NEW] offline task replay harness
│   ├── mod.rs
│   ├── harness.rs
│   └── tasks/                     ← .json task definitions
│
├── provider/                      ← (keep)
│   ├── mod.rs
│   ├── gemini.rs
│   └── groq.rs
│
├── tools/                         ← (keep)
│   └── ...
│
├── traits/                        ← (keep)
│   ├── mod.rs
│   ├── provider.rs
│   └── tool.rs
│
└── types/                         ← (keep)
    ├── mod.rs
    ├── approval.rs
    ├── error.rs
    ├── event.rs
    └── message.rs
```

---

## Summary of Changes

| Action | Item |
|---|---|
| **NEW dir** | `agentic_loop/` — `engine.rs`, `tool_dispatch.rs`, `approval.rs`, `cancellation.rs` |
| **RENAME** | `orchestrator/loop_engine.rs` → `agentic_loop/engine.rs` |
| **RENAME** | `orchestrator/context.rs` → `context/pruner.rs` |
| **RENAME** | `orchestrator/state.rs` → `orchestrator/session.rs` |
| **NEW dir** | `context/` — `workspace.rs`, `git_context.rs` |
| **NEW dir** | `skills/` — skill trait + 3 built-ins |
| **NEW dir** | `subagent/` — mini-loop runner |
| **NEW dir** | `verification/` — build/test checker + retry |
| **NEW dir** | `memory/` — cross-session summarizer |
| **NEW dir** | `evaluation/` — offline harness + task definitions |
| **KEEP** | `provider/`, `tools/`, `traits/`, `types/` |

> [!IMPORTANT]
> Recommended implementation order:
> 1. **Refactor first** — extract `agentic_loop/` from `loop_engine.rs` (no behaviour change, just structure)
> 2. **verification/** — closes the fail→retry loop (biggest impact)
> 3. **skills/** — per-task instructions (biggest quality improvement)
> 4. **context/workspace.rs** — injects open file context into the prompt
> 5. `subagent/`, `memory/`, `evaluation/` — later, as needed


```
crates/agent/src/
├── lib.rs
│
├── context/                       ← [RENAME from orchestrator/context.rs]
│   ├── mod.rs
│   ├── pruner.rs                  ← history pruning, sanitize_tool_result (moved here)
│   ├── workspace.rs               ← [NEW] reads open files, symbols, current dir for injection
│   └── git_context.rs             ← [NEW] injects git status/diff into system prompt
│
├── orchestrator/
│   ├── mod.rs
│   ├── config.rs                  ← (keep)
│   ├── planner.rs                 ← [NEW] picks skills/strategy before loop runs
│   ├── loop_engine.rs             ← (keep, rename agentic_loop.rs → loop_engine.rs already done)
│   └── session.rs                 ← [RENAME from state.rs] conversation export/import
│
├── skills/                        ← [NEW] per-task instruction bundles
│   ├── mod.rs
│   ├── skill.rs                   ← Skill trait (name, system_prompt, tools_override)
│   ├── coding.rs                  ← "write/edit code" skill
│   ├── debugging.rs               ← "diagnose and fix" skill
│   └── explain.rs                 ← "explain codebase" skill
│
├── subagent/                      ← [NEW] mini-loop for isolated sub-tasks
│   ├── mod.rs
│   └── runner.rs                  ← SpawnedAgent: runs a sub-task, returns result to parent
│
├── verification/                  ← [NEW] post-loop checks
│   ├── mod.rs
│   ├── runner.rs                  ← runs build/lint/test, returns Pass | Fail(reason)
│   └── strategies.rs              ← cargo build, cargo test, eslint, etc.
│
├── memory/                        ← [NEW] cross-session summaries
│   ├── mod.rs
│   └── summarizer.rs              ← compresses old conversations into a short summary
│
├── evaluation/                    ← [NEW] offline task replay harness
│   ├── mod.rs
│   ├── harness.rs                 ← runs a fixed task set through the loop
│   └── tasks/                     ← .json task definitions
│
├── provider/                      ← (keep as-is)
│   ├── mod.rs
│   ├── gemini.rs
│   └── groq.rs
│
├── tools/                         ← (keep as-is)
│   ├── ...
│
├── traits/                        ← (keep, may add Skill + Verifier traits)
│   ├── mod.rs
│   ├── provider.rs
│   └── tool.rs
│
└── types/                         ← (keep, may add VerificationResult)
    ├── mod.rs
    ├── approval.rs
    ├── error.rs
    ├── event.rs
    └── message.rs
```

---

## Summary of Changes

| Action | Item |
|---|---|
| **RENAME** | `orchestrator/context.rs` → `context/pruner.rs` |
| **RENAME** | `orchestrator/state.rs` → `orchestrator/session.rs` |
| **NEW dir** | `context/` with `workspace.rs`, `git_context.rs` |
| **NEW dir** | `skills/` with skill trait + 3 built-in skills |
| **NEW dir** | `subagent/` with `runner.rs` |
| **NEW dir** | `verification/` with `runner.rs` + `strategies.rs` |
| **NEW dir** | `memory/` with `summarizer.rs` |
| **NEW dir** | `evaluation/` with `harness.rs` + task definitions |
| **KEEP** | `orchestrator/loop_engine.rs`, `provider/`, `tools/`, `traits/`, `types/` |

> [!IMPORTANT]
> The three highest-impact additions in order are:
> 1. **verification/** — closes the fail→retry loop (core of the diagram)
> 2. **skills/** — gives the agent task-specific instructions (biggest quality improvement)
> 3. **context/workspace.rs** — injects open file + symbols into prompt (improves relevance)
>
> `subagent/`, `memory/`, and `evaluation/` are lower priority and can come later.
