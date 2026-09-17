# Cognitive Loop Architecture: Granular Inspection, Deep Deliberation & Progressive Surgical Editing

This document specifies the end-to-end design and technical implementation for Z.E.N.E.'s **Cognitive Engineering Loop** in the Zenthree code editor. It details how the agent investigates codebases slice-by-slice, reasons out loud through collapsible thought blocks, and progressively performs multi-location surgical modifications.

---

## 1. Executive Summary & The Problem

### The Blind-Batching Anti-Pattern in Traditional Coding Agents
Most coding assistants attempt to edit complex codebases using monolithic batching:
1. They ingest a high-level prompt and immediately attempt to generate or rewrite all changes in a single giant hallucinated pass.
2. They do not pause to visually inspect the exact line numbers or surrounding context on disk.
3. If a single file requires modifications across 4 different locations (e.g. imports, state hooks, business logic, and UI rendering), an ungrounded model frequently introduces syntax errors, misaligns indentation, or silently deletes critical existing logic.

### The Cognitive Engineering Solution ("See → Think → Edit → Reflect")
Instead of blind generation, Z.E.N.E. operates on a disciplined cognitive pipeline:
- **See Before Touching**: Before touching any file, the agent inspects targeted line slices (`#L1058-1085`).
- **Reason Out Loud**: Before and between edits, the agent articulates an internal chain-of-thought monologue (`Thought for Xs`).
- **Progressive Surgical Edits**: For files with multiple edit locations, the agent executes changes sequentially, verifying consistency after each step.
- **Hierarchical UI Transparency**: The entire process is exposed to the developer through clean, collapsible accordions (`Worked for 42s`, `Explored 2 files, 1 search`, `Thought for 2s`).

---

## 2. Visual Architecture & UI Representation

The UI in the AI chat panel mirrors the clean, granular disclosure pattern found in state-of-the-art developer environments:

```
┌─────────────────────────────────────────────────────────────────────────┐
│ User: "Fix the layout padding and add theme toggle in ai_panel.rs"       │
└─────────────────────────────────────────────────────────────────────────┘
  │
  ▼
┌─────────────────────────────────────────────────────────────────────────┐
│ ▾ Worked for 42s                                                        │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │ ▾ Explored 2 files, 1 search                                    │   │
│   │   • Searched `active_tab` (47 results)                          │   │
│   │   • Analyzed 🦀 ai_panel.rs #L1058-1085                         │   │
│   │   • Analyzed 🦀 ai_panel.rs #L1145-1185                         │   │
│   │   ▸ Thought for 1s                                              │   │
│   │   • Analyzed 🦀 ai_panel.rs #L1370-1425                         │   │
│   │   ▾ Thought for 2s                                              │   │
│   │     "Initial assessment suggests examining the layout padding  │   │
│   │      and state binding. I need to update imports first, then    │   │
│   │      wire up the theme toggle without affecting the toolbar."   │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│   • Edited 🦀 ai_panel.rs +41 -10 [✕]                                   │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │ ▾ Explored 1 file                                               │   │
│   │   • Analyzed 🦀 main_buffer.rs #L2150-2200                      │   │
│   │   ▸ Thought for 1s                                              │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│   • Edited 🦀 main_buffer.rs +12 -3 [✕]                                 │
└─────────────────────────────────────────────────────────────────────────┘
  │
  ▼
Markdown summary explaining what was accomplished.
```

---

## 3. The 4-Stage Cognitive Loop Mechanics

```
                  ┌─────────────────────────────────────┐
                  │          1. SEE THE CODE            │
                  │  read_file(path, start, end)        │
                  │  UI: "Analyzed file.rs #L10-40"     │
                  └──────────────────┬──────────────────┘
                                     │
                                     ▼
                  ┌─────────────────────────────────────┐
                  │         2. THINK & DELIBERATE       │
                  │  Internal reasoning monologue       │
                  │  UI: "Thought for 2s >"             │
                  └──────────────────┬──────────────────┘
                                     │
                                     ▼
                  ┌─────────────────────────────────────┐
                  │         3. SURGICALLY EDIT          │
                  │  SEARCH/REPLACE block for Part N    │
                  │  UI: "Edited file.rs +4 -2"         │
                  └──────────────────┬──────────────────┘
                                     │
                                     ▼
                  ┌─────────────────────────────────────┐
                  │         4. REFLECT & ADVANCE        │
                  │  Verify result & check next section │
                  │  More parts? -> Return to Step 1    │
                  └─────────────────────────────────────┘
```

### Stage 1: Granular Code Slicing (`Analyzed #Lstart-end`)
- Instead of reading 2,000 lines into context, the agent queries specific ranges:
  `{"tool": "read_file", "path": "ai_panel.rs", "start_line": 1058, "end_line": 1085}`
- In the UI, this is badged as:
  `Analyzed [Rust icon] ai_panel.rs #L1058-1085`
- Clicking this badge jumps the active editor directly to those lines.

### Stage 2: Deep Deliberation Blocks (`Thought for Xs`)
- The model outputs an explicit thought monologue explaining:
  - What it observed in the analyzed lines.
  - Potential side-effects on types, imports, or state.
  - The exact strategy for the upcoming modification.
- In the UI, this renders as a collapsible `Thought for Xs >` disclosure block.
- Clicking expands it (`Thought for Xs v`), revealing the raw formatted thought text.

### Stage 3: Progressive Multi-Location Editing
When a single file has multiple modification points (e.g. 4 parts):
- **Part 1 (Imports)**: Analyze imports `#L1-20` $\rightarrow$ Think $\rightarrow$ Emit surgical patch $\rightarrow$ Reflect.
- **Part 2 (State/Hooks)**: Analyze component declaration `#L40-65` $\rightarrow$ Think $\rightarrow$ Emit surgical patch $\rightarrow$ Reflect.
- **Part 3 (Handlers)**: Analyze callbacks `#L110-145` $\rightarrow$ Think $\rightarrow$ Emit surgical patch $\rightarrow$ Reflect.
- **Part 4 (Rendering/JSX)**: Analyze UI return `#L200-240` $\rightarrow$ Think $\rightarrow$ Emit surgical patch $\rightarrow$ Reflect.

### Stage 4: Progressive Execution & Rollback Safety
- Each edit is staged into `pending_modifications` with `original_code` and `proposed_code`.
- The user can inspect the live red/green line diff in the buffer and reject single files (`✕`) or accept all.

---

## 4. Implementation Blueprint in Zenthree

### Step 1: Extend Event Streaming (`crates/ai/src/types/events.rs`)

Add structured event variants to represent granular analysis and thought blocks:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    /// Granular line-slice analysis event
    FileAnalyzed {
        file_path: String,
        start_line: usize,
        end_line: usize,
    },

    /// Search query execution event
    SearchExecuted {
        query: String,
        result_count: usize,
    },

    /// Deliberation / Chain-of-thought block
    ThoughtBlock {
        content: String,
        duration_secs: u64,
    },

    /// Progressive surgical file modification
    PendingModifications {
        summaries: Vec<EditedFileSummary>,
    },

    /// Tool execution lifecycle
    ToolStarted { tool: String, summary: String },
    ToolFinished { tool: String, ok: bool, output_snippet: String, duration_ms: u64 },

    /// Final goal completion
    Completed { summary: String },
    Error { error: String },
}
```

---

### Step 2: Update Chat Message Data Model (`crates/zenthree/src/state.rs`)

Store structured exploration steps inside each assistant message:

```rust
#[derive(Debug, Clone)]
pub enum ActivityItem {
    Search { query: String, results: usize },
    Analysis { file_name: String, start_line: usize, end_line: usize },
    Thought { content: String, duration_secs: u64, is_expanded: bool },
    ToolCall { tool: String, summary: String, ok: bool },
}

#[derive(Debug, Clone)]
pub struct ExplorationGroup {
    pub title: String, // e.g. "Exploring 2 files, 1 search"
    pub items: Vec<ActivityItem>,
    pub is_expanded: bool,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub is_user: bool,
    pub content: String,
    pub exploration_groups: Vec<ExplorationGroup>,
    pub edited_files: Vec<EditedFileSummary>,
    pub total_duration_secs: u64,
    pub is_working: bool,
}
```

---

### Step 3: Loop Engine Protocol (`crates/ai/src/orchestrator/loop_engine.rs`)

The loop engine intercepts tools and thought markers during execution:

```rust
// 1. When `read_file` executes with start_line and end_line:
(self.event_sender)(AgentEvent::FileAnalyzed {
    file_path: path.to_string(),
    start_line,
    end_line,
});

// 2. When `search_code` executes:
(self.event_sender)(AgentEvent::SearchExecuted {
    query: query.to_string(),
    result_count: results.len(),
});

// 3. When the LLM emits a <thought> ... </thought> or Thinking block:
(self.event_sender)(AgentEvent::ThoughtBlock {
    content: thought_text,
    duration_secs: elapsed_secs,
});
```

---

### Step 4: System Prompt Enforcement (`crates/ai/src/context/prompts.rs`)

Instruct the model to follow the strict cognitive protocol:

```markdown
DISCIPLINED COGNITIVE EDITING PROTOCOL:
When modifying files:
1. NEVER guess line numbers or surrounding code. Always call `read_file` with line bounds to SEE the code first.
2. For each modification location in a file:
   - Articulate your thinking in a `<thought>` block: analyze what needs changing and why.
   - Emit the exact surgical `<<<<<<< SEARCH ... ======= ... >>>>>>> REPLACE` block for that location.
   - Reflect on the change before moving to the next location.
3. Proceed sequentially down the file until all parts are complete.
```

---

### Step 5: Chat Bubble UI Rendering (`crates/zenthree/src/ui_layout/ai_panel.rs`)

Render the hierarchical disclosures in the chat bubble:

```rust
// 1. Top-level "Worked for Xs" disclosure (when finished)
// 2. Accordion "Explored N files, N searches"
// 3. Render items:
//    - "Searched <query> <count> results"
//    - "Analyzed <icon> <file> #Lstart-end"
//    - "Thought for <duration>s > / v" (clicking toggles is_expanded)
//      If expanded: render thought content in muted box
// 4. Render "Edited <icon> <file> +added -removed [✕]"
// 5. Render markdown response content
```

---

## 5. Summary of Benefits

1. **Eliminates Code Clobbering**: The agent never replaces full files blindly; it inspects and patches only the required lines.
2. **Transparent Developer Visibility**: The developer can see exactly what files were opened, what line ranges were analyzed, and what the agent was thinking at every second.
3. **Resilient Multi-Location Edits**: Even complex refactors touching 4–10 locations in a single file succeed because each part is verified in sequence.
4. **Instant Single-File Rollback**: Every edited file provides a direct reject (`✕`) button to revert changes immediately.
