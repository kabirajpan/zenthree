# AI Thinking Architecture

## Overview

This document describes the staged, intent-first agent architecture for ZENE.

Core principles:
- **Don't dump everything on the LLM upfront** — be smart about what gets sent and when
- **Speed + quality** — pre-select the right tools, not all tools
- **Search as fallback** — LLM can discover more tools/skills mid-task if needed

---

## The Problem with the Current Approach

Right now the agent sends all tools and skills on every message, blindly:

```
User message → send ALL 15 tool schemas + ALL skills → agentic loop
                ▲
                ❌ ~3,750 tokens wasted per message
                ❌ LLM confused by irrelevant tools
                ❌ No intent understanding before acting
```

---

## The New Architecture

### Two Paths

```
PRIMARY PATH (fast, pre-selected)       FALLBACK PATH (discovery)
─────────────────────────────────       ─────────────────────────
Used on every message                   Used mid-task when needed

Intent → pre-select 3-5 tools          LLM: "I need something to run commands"
→ send full schemas                     search(query:"terminal", in:"tools")
→ agentic loop starts                   → discovers run_terminal
                                        → requests full schema
No extra round trips                    → continues
```

---

## Full Flow Diagram

```
User message
      │
      ▼
 ┌────────────────────────────┐
 │  Step 1-2: Intent Phase    │
 │  LLM (no tools, no skills) │
 │  "what does user want?"    │
 │  "what kind of task?"      │
 └────────────┬───────────────┘
              │  intent understood
              │  e.g. "debug/fix Rust code"
              ▼
 ┌────────────────────────────┐
 │  Step 3: Agent Pre-selects │  ← rule-based, instant, no LLM call
 │                            │
 │  maps intent → tool subset │
 │  "debug" →                 │
 │    search                  │
 │    read_file               │
 │    edit_file               │
 │    get_diagnostics         │
 │    run_terminal            │
 └────────────┬───────────────┘
              │
              ▼
 ┌────────────────────────────┐
 │  Step 4: Send Full Schemas │
 │  only for selected tools   │
 │  ~5 × 300 = ~1,500 tokens  │
 │  (not 15 × 250 = 3,750)    │
 └────────────┬───────────────┘
              │
              ▼
 ┌────────────────────────────┐
 │  Step 5: Agentic Loop      │
 │  LLM acts immediately      │
 │  uses pre-loaded tools     │
 │                            │
 │  If LLM needs more:        │
 │  search(in:"tools+skills") │ ← fallback discovery
 │  → finds extra tools       │
 │  → requests schema         │
 │  → cached for session      │
 └────────────────────────────┘
```

---

## Step 3 — Agent Pre-selection Rules

The agent maps intent keywords → tool subsets. This is fast, rule-based, no LLM call needed.

```
Intent: "fix bug" / "debug" / "error"
  → search, read_file, edit_file, get_diagnostics, run_terminal

Intent: "add feature" / "implement" / "create"
  → search, read_file, edit_file, write_file, get_diagnostics

Intent: "understand" / "explore" / "what does" / "explain"
  → search, read_file, get_project_structure, list_directory

Intent: "refactor" / "rename" / "move"
  → search, read_file, edit_file, rename_file, get_diagnostics

Intent: "test" / "verify" / "run"
  → run_terminal, get_diagnostics, read_file

Intent: "review" / "check" / "diff"
  → git_status, git_diff, get_diagnostics, read_file

Intent: unknown / complex
  → send all essential tools (fallback to current behavior)
```

Skills are handled the same way — relevant skill name injected as part of the system context for the agentic loop.

---

## Session Cache

Once the LLM requests a full schema via `search(in:"tools")` mid-task:

```
First request:
  LLM: search(query:"terminal", in:"tools")
  → agent returns: run_terminal : Execute a shell command
  LLM: "give me full schema for run_terminal"
  → agent sends full schema
  → cache: { run_terminal: full_schema }

Next time in same session:
  → full schema sent directly, no re-request
  → saves one round trip + ~300 tokens
```

Cache is **per-session only** — resets on new conversation. Schemas can change between sessions.

---

## The Search Tool as Fallback

The universal `search` tool serves as the LLM's self-discovery mechanism mid-task:

```
search(
  query:  string   ← what to look for
  in:     string   ← combine any: tools | skills | folders | files | content
                     default: "all" (searches everything)
  path:   string   ← scope for filesystem searches
  ext:    string   ← file extension filter
  regex:  bool     ← treat query as regex
  case:   bool     ← case sensitive
)
```

Typical fallback uses:

```
LLM stuck? → search(query:"rename move file", in:"tools")
Need a skill? → search(query:"review code", in:"skills")
Need code? → search(query:"memory_load", in:"content", ext:"rs")
Find a module? → search(query:"auth", in:"folders+files")
```

See [`Tools_Search.md`](Tools_Search.md) for the full search tool specification.

---

## Token Budget Comparison

| Approach | Token cost per message |
|----------|----------------------|
| Current (all schemas always) | ~3,750 tokens |
| New (pre-selected 3-5 tools) | ~1,125 tokens |
| Savings | ~2,600 tokens (~70% less) |

On a 100-message conversation: **~260,000 tokens saved** in tool schema overhead alone.

---

## Key Design Decisions

| Decision | Choice | Reason |
|----------|--------|--------|
| Tool discovery | Agent pre-selects (not LLM) | No extra round trips → speed |
| How many tools sent | 3-5 relevant | LLM focuses better, acts faster |
| Lazy loading | Dropped as primary | Adds round trips, hurts speed |
| Search tool | Universal fallback discovery | LLM finds more tools if needed |
| Skills | Injected per intent | Same pre-selection logic |
| Session cache | Per-session only | Schemas may change across sessions |
| Fallback | Send all essentials if intent unclear | Never leave LLM with no tools |

---

## What Changes vs Current

| | Current | New |
|-|---------|-----|
| Tool schemas sent | All 15, every message | 3-5 pre-selected per intent |
| Skills | Full summary in system msg | Injected per intent |
| Intent phase | None | Explicit Step 1-2 before tools |
| Tool routing | None | Agent maps intent → tools |
| Discovery | None | `search(in:"tools+skills")` fallback |
| Session cache | None | Caches expanded schemas mid-session |
| Token cost | ~3,750/msg | ~1,125/msg |
