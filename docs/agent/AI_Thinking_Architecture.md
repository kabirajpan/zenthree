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

## The New Architecture: LLM Intent & On-Demand Tools

### Core Paradigm

Instead of blindly assuming every user prompt requires code modifications and dumping all 15 tool schemas:
1. **The LLM decides the intention first**:
   - Handles typos (*"chck auth eror"*) and implicit meanings naturally.
   - Works regardless of whether files are currently open in editor tabs.
   - Recognizes when the user wants to **discuss / brainstorm / ask advice** vs take **action**.
2. **Tools are delivered on-demand**:
   - **Discussion Mode:** **0 tools loaded**. Pure conversational response. Zero token waste.
   - **Exploration Mode:** **2 tools loaded** (`search`, `read_file`).
   - **Action Mode:** **3–5 tools loaded** (`search`, `read_file`, `edit_file`, `get_diagnostics`, etc.).
3. **Mid-task discovery**: If scope expands, LLM calls `search(in: "tools+skills")` to load more schemas dynamically into the session cache.

---

## Full Flow Diagram

```
                 User message (Any phrasing / typos / unopened files)
                                          │
                                          ▼
                             ┌─────────────────────────┐
                             │  Step 1: Intent Phase   │
                             │  LLM evaluates goal:    │
                             │  Discuss vs Explore vs  │
                             │  Action                 │
                             └────────────┬────────────┘
                                          │
               ┌──────────────────────────┼──────────────────────────┐
               │                          │                          │
               ▼                          ▼                          ▼
      [Intent: Discuss]         [Intent: Explore]          [Intent: Action]
      • Architecture advice     • "Where is auth?"         • "Fix this error"
      • "Should I use SQLite?"  • "What files handle X?"   • "Refactor module"
      • General Q&A / chat      • "Explain this logic"     • "Implement feature"
               │                          │                          │
               ▼                          ▼                          ▼
      ┌─────────────────┐       ┌─────────────────┐       ┌──────────────────┐
      │  0 Tools Loaded │       │ 2 Tools Loaded  │       │ 4-5 Tools Loaded │
      │  Pure chat text │       │ search,read_file│       │search,read,edit..│
      └─────────────────┘       └─────────────────┘       └──────────────────┘
               │                          │                          │
               ▼                          ▼                          ▼
       Natural response            Focused answer             Surgical edits
      (0 tokens wasted)          (600 tokens used)         (1,200 tokens used)
                                          │                          │
                                          └───────────┬──────────────┘
                                                      │
                                                      ▼
                                         ┌───────────────────────────┐
                                         │ Need more tools mid-task? │
                                         │ search(in: "tools")       │
                                         │ → expands session cache   │
                                         └───────────────────────────┘
```

---

## The 3 Intent Categories

### 1. Discussion Mode (0 Tools Loaded)
When the user wants to talk, brainstorm, or compare options:
* **User Examples:**
  - *"How should we design the database schema for multi-tenancy?"*
  - *"Can you explain the difference between Arc and Rc?"*
  - *"What are your thoughts on this architecture?"*
* **Tools Loaded:** **None (0 tools)**
* **Output:** Fast, natural, high-quality markdown discussion.
* **Token Overhead:** **0 tokens.**

### 2. Exploration Mode (2 Tools Loaded)
When the user wants to understand existing project structure or find code:
* **User Examples:**
  - *"Where is the JWT verification implemented?"*
  - *"What files are responsible for billing?"*
* **Tools Loaded:** `search`, `read_file`
* **Token Overhead:** **~600 tokens.**

### 3. Action Mode (3–5 Tools Loaded On-Demand)
When the user requests concrete code modifications, bug fixes, or running commands:
* **Bug Fix / Debugging:**
  `search`, `read_file`, `edit_file`, `get_diagnostics`, `run_terminal`
* **Feature Implementation:**
  `search`, `read_file`, `edit_file`, `write_file`, `get_diagnostics`
* **Refactoring:**
  `search`, `read_file`, `edit_file`, `rename_file`, `get_diagnostics`
* **Testing / Verification:**
  `run_terminal`, `get_diagnostics`, `read_file`
* **Review / Quality:**
  `git_status`, `git_diff`, `get_diagnostics`, `read_file`

---

## Session Cache

Once the LLM requests or loads a full schema mid-task:

```
First request:
  LLM: search(query:"terminal", in:"tools")
  → agent returns: run_terminal : Execute a shell command
  LLM: "give me full schema for run_terminal"
  → agent sends full schema
  → cache: { run_terminal: full_schema }

Next time in same session:
  → full schema sent directly, no re-request
  → saves round trips and token overhead
```

Cache is **per-session only** — resets on new conversation.

---

## The Search Tool as Fallback

The universal `search` tool serves as the LLM's self-discovery mechanism mid-task across 6 scopes:

```
search(
  query:  string   ← what to look for
  in:     string   ← combine any: tools | skills | folders | files | symbols | content
                     default: "all" (searches everything)
  path:   string   ← scope for filesystem searches
  ext:    string   ← file extension filter
  regex:  bool     ← treat query as regex
  case:   bool     ← case sensitive
)
```

Typical fallback uses:

```
LLM stuck?    → search(query:"rename move file", in:"tools")
Need a skill? → search(query:"review code", in:"skills")
Need symbol?  → search(query:"verify_token", in:"symbols")
Need code?    → search(query:"memory_load", in:"content", ext:"rs")
Find folder?  → search(query:"auth", in:"folders+files")
```

See [`Tools_Search.md`](Tools_Search.md) for the full search tool specification.

---

## Intent Execution & Thinking Protocol

### How the Model Thinks and Decides

Modern reasoning models (Gemini 2.0 Flash Thinking, Claude 3.7 Sonnet Thinking) generate a structured internal `<thinking>` block before emitting output or tool calls.

The ZENE agent leverages this native thinking capability to make intent evaluation zero-friction:

```markdown
<thinking>
1. User Intent Evaluation:
   - Query: "the auth token refresh fails with 401 sometimes"
   - Analysis: User is reporting an intermittent authentication bug.
   - Intent Category: ACTION (Debugging / Bug Fix).
   - Needs: Search for token refresh logic, read implementation, diagnose.
2. Tool Strategy:
   - Start by locating token refresh handlers using symbols search.
</thinking>

search(query: "refresh_token", in: "symbols+content")
```

### The 3 Execution Pathways

#### Pathway 1: Discussion / Brainstorming
- **Trigger:** User asks architectural questions, trade-offs, language concepts, or opinions.
- **Tools Attached:** **0 tools.**
- **Model Behavior:** Thinks through the problem and immediately streams clean, rich markdown without calling any tool.
- **Latency & Token Savings:** Instant first-token latency, **3,750 tokens saved** per turn.

#### Pathway 2: Exploration
- **Trigger:** User asks "where is X?", "which module handles Y?", or "explain how the router works".
- **Tools Attached:** `search`, `read_file`.
- **Model Behavior:** Uses `search(in: "symbols")` or `read_file(peek: true)` to locate and understand code, then answers the user clearly with file links.

#### Pathway 3: Action & Execution
- **Trigger:** User asks to fix, implement, refactor, test, or verify code.
- **Tools Attached:** Pre-loaded 3–5 action tools (`search`, `read_file`, `edit_file`, `get_diagnostics`, `run_terminal`).
- **Model Behavior:** Surgical edit loop. Every edit is validated via Tree-sitter and compiler diagnostics before completion.

---

## UI Panel Integration & Trajectory

In Zenthree (`crates/zenthree/src/ui_layout/ai_panel.rs`), the user's experience is mirrored dynamically:

```
[User Message]: "why does token refresh fail?"
   │
   ▼
[Accordion]: "Thinking Process (1.8s)"
   ├── Model internal thoughts: "Analyzing token expiry and refresh flow..."
   └── Intent: Action (Debug)
   │
   ▼
[Trajectory Group]: "Exploration"
   ├── Tool call: search("refresh_token", in: "symbols")  -> src/auth/token.rs:89
   └── Tool call: read_file("src/auth/token.rs", symbol: "refresh_token")
   │
   ▼
[Assistant Response]: "The refresh token fails because the expiry check..."
```

---

## Token Budget Comparison

| Approach | Schema Tokens per Turn | Tool Noise for LLM | Latency Impact |
|----------|------------------------|---------------------|----------------|
| **Current (All 15 schemas always)** | ~3,750 tokens | High (confusing) | Slow (bloated prompt) |
| **Discussion Mode** | **0 tokens** | Zero | Ultra-fast (instant stream) |
| **Exploration Mode** | ~600 tokens | Low | Fast |
| **Action Mode** | ~1,200 tokens | Optimal | Fast, focused |

Across an average 50-turn coding session: **~120,000 to 180,000 tokens saved**.

---

## Key Design Decisions

| Decision | Choice | Reason |
|----------|--------|--------|
| **Intent Decider** | LLM (Native Thinking) | Handles typos, slang, implicit requests, and unopened files naturally |
| **Discussion Mode** | 0 tools loaded | Zero tokens wasted when user just wants to talk or brainstorm |
| **Action Mode** | 3–5 targeted tools | Prevents LLM context clutter while giving full execution power |
| **Fallback Discovery** | Universal `search` tool | If scope expands mid-task, LLM discovers extra tools dynamically |
| **Session Cache** | In-memory per conversation | Tools discovered mid-task remain cached for subsequent turns |
| **Safety Shield** | Tree-sitter pre-flight | Catches syntax errors in 0.5ms before slow compilers run |

---

## Summary of Changes vs Old System

| Area | Old System | New AI Thinking Architecture |
|------|------------|-----------------------------|
| **Tool Schemas Sent** | All 15 tools every single turn | On-demand: 0 (Discuss), 2 (Explore), 4-5 (Action) |
| **Chat / Discussion** | Loaded all tools even for "hello" | 0 tools loaded, instant markdown stream |
| **Typo Handling** | Rigid keyword failure | LLM semantic understanding |
| **Discovery** | None | `search(in: "tools+skills")` fallback |
| **Token Overhead** | ~3,750 tokens / turn | 0 to 1,200 tokens / turn (up to 90% savings) |
