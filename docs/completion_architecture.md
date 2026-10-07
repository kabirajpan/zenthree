# Zenthree Code Suggestion & Completion Architecture

> A high-performance, multi-tier code completion engine combining **Zed's 0ms instant buffer speed**, **VS Code's type-aware LSP precision**, and **JetBrains' smart fuzzy/postfix capabilities**.

---

## 1. Executive Summary & Design Principles

Modern developers expect code completion that is **instantaneous**, **accurate**, and **unobtrusive**. Relying exclusively on an LSP causes noticeable latency (100–300ms) or complete failure if no language server binary is installed.

Zenthree implements a **Hybrid 3-Tier Cascade Engine**:

1. **⚡ Tier 1: Instant Local Tier (Zed-style, 0ms latency)**
   - Local buffer token scanner extracts identifiers from all open editor tabs.
   - Built-in language keyword dictionaries (Rust, TS/JS, Python, Go, C/C++).
   - Zero-allocation string scanning directly on the frame; guarantees suggestions always appear.

2. **🔍 Tier 2: Async LSP Enrichment (VS Code-style)**
   - Non-blocking background JSON-RPC query to `rust-analyzer`, `pyright`, `gopls`, or `tsserver`.
   - When the response arrives, seamlessly merges rich types, methods, documentation, and signatures.

3. **🧠 Tier 3: Fuzzy, CamelHump & Postfix Intelligence (JetBrains-style)**
   - **CamelHump matching:** typing `HM` matches `HashMap`, `fnd` matches `find_node_depth`.
   - **Postfix templates:** typing `expr.match`, `expr.if`, `expr.dbg` auto-expands into structured syntax.
   - **Clean range replacement:** cleanly deletes the typed prefix so `obj.to_` + `to_string` produces `obj.to_string`, never `obj.to_to_string`.

---

## 2. System Architecture & Data Flow

```text
 ┌────────────────────────────────────────────────────────────────────────┐
 │                              User Types Keystroke                      │
 └───────────────────────────────────┬────────────────────────────────────┘
                                     │
           ┌─────────────────────────┴────────────────────────┐
           ▼                                                  ▼
 ┌──────────────────────────────┐                   ┌──────────────────────────────┐
 │   Tier 1: Instant Engine     │                   │     Tier 2: Async LSP        │
 │   (Local Buffer + Keywords)  │                   │     (Language Server RPC)    │
 └──────────────┬───────────────┘                   └──────────────┬───────────────┘
                │ [0ms synchronous]                                │ [50-150ms async]
                ▼                                                  ▼
 ┌──────────────────────────────┐                   ┌──────────────────────────────┐
 │   Fuzzy & CamelHump Filter   │                   │    LSP Results Merge / Rank  │
 │   (Score & Rank Candidates)  │                   │    (Elevate typed methods)   │
 └──────────────┬───────────────┘                   └──────────────┬───────────────┘
                │                                                  │
                └─────────────────────────┬────────────────────────┘
                                          ▼
                         ┌──────────────────────────────────┐
                         │   CompletionCoordinator State    │
                         │   • active: bool                 │
                         │   • prefix: String               │
                         │   • items: Vec<CompletionItem>   │
                         │   • selected_idx: usize          │
                         └────────────────┬─────────────────┘
                                          ▼
                         ┌──────────────────────────────────┐
                         │    Floating Popup UI Rendering   │
                         │    (Nerd Font Icons + Types)     │
                         └────────────────┬─────────────────┘
                                          │
                    [Tab] / [Enter]       │     [Escape] / Navigation
                                          ▼
                         ┌──────────────────────────────────┐
                         │  Clean Range Insertion into Rope │
                         │  (Deletes prefix, inserts text)  │
                         └──────────────────────────────────┘
```

---

## 3. Modular Codebase Structure

To keep the codebase maintainable and decoupled, the completion system lives in a dedicated module directory:

```text
crates/zenthree/src/completion/
├── mod.rs                  # CompletionCoordinator: orchestrates state, triggers & insertion
├── buffer_indexer.rs       # Scans active buffers for identifiers & symbols (Tier 1)
├── keywords.rs             # Language keywords database (Rust, TS/JS, Python, Go, C/C++)
├── fuzzy.rs                # Fuzzy & CamelHump matcher with scoring (Tier 3)
├── postfix.rs              # JetBrains-style postfix expansions (.if, .match, .dbg, .unwrap)
└── ui.rs                   # Floating UI popup, Nerd Font icons, clipping & mouse interaction
```

---

## 4. Component Deep Dive

### 4.1. Instant Buffer Indexer (`buffer_indexer.rs`)
- Scans lines within the active editor buffer using an ASCII/UTF-8 identifier rule: `[a-zA-Z_][a-zA-Z0-9_]*`.
- Dedupes results using a small `HashSet<&str>`.
- Excludes the word currently being typed under the cursor to prevent self-matching.
- Operates under **< 0.5ms** for buffers up to 20,000 lines.

### 4.2. Language Keywords (`keywords.rs`)
Provides static keyword and snippet lists per language extension:
- **Rust (`.rs`):** `fn`, `let`, `mut`, `struct`, `enum`, `impl`, `match`, `pub`, `use`, `trait`, `async`, `await`, `where`, `continue`, `break`, `return`
- **TypeScript / JavaScript (`.ts`, `.tsx`, `.js`, `.jsx`):** `const`, `let`, `function`, `export`, `import`, `return`, `async`, `await`, `class`, `interface`, `type`
- **Python (`.py`):** `def`, `class`, `import`, `from`, `return`, `yield`, `self`, `async`, `await`, `with`, `lambda`
- **Go (`.go`):** `func`, `package`, `import`, `type`, `struct`, `interface`, `chan`, `go`, `select`, `defer`
- **C / C++ (`.c`, `.cpp`, `.h`, `.hpp`):** `auto`, `template`, `namespace`, `class`, `struct`, `constexpr`, `inline`, `nullptr`

### 4.3. Fuzzy & CamelHump Matcher (`fuzzy.rs`)
Given a query and a candidate string:
1. **Exact Prefix Match (Score: 1000):** `to_` matches `to_string`.
2. **CamelHump Match (Score: 800):** `HM` matches `HashMap`, `FSR` matches `FileStreamReader`.
3. **Sub-word / Fuzzy Match (Score: 500 - distance):** `fnd` matches `find_node_depth`.
4. Ranks results so exact prefix and LSP-typed symbols appear at the top.

### 4.4. Postfix Snippets (`postfix.rs`)
Detects triggers after a dot (`.`):
- `expr.match` ➔ expands to:
  ```rust
  match expr {
      Ok(val) => { █ },
      Err(err) => {},
  }
  ```
- `expr.if` ➔ `if expr { █ }`
- `expr.dbg` ➔ `dbg!(expr)`
- `expr.unwrap` ➔ `expr.unwrap()`
- `expr.not` ➔ `!expr`

### 4.5. Clean Range Replacement (`mod.rs`)
- **Bug Prevention:** Instead of blindly calling `editor.insert()`, calculates:
  ```rust
  let prefix_len = current_word_prefix.chars().count();
  for _ in 0..prefix_len {
      tab.editor.backspace();
  }
  tab.editor.insert(chosen_completion, ...);
  ```
- Guarantees `input.to_` + `to_string` results in `input.to_string`.

---

## 5. UI & Interaction Design

### Visual Layout (Nerd Font Glyph Icons)

```text
 12 │ fn process_data(input: &str) -> Result<String, Error> {
 13 │     let result = input.to_█
                          ┌────────────────────────────────────────────────────────┐
                          │ 󰊕 to_string()            fn(&self) -> String         │ ◄── Highlighted
                          │ 󰊕 to_lowercase()         fn(&self) -> String         │
                          │ 󰊕 to_uppercase()         fn(&self) -> String         │
                          │ 󰡱 to_mut(&mut self)      fn(&mut self) -> &mut str   │
                          │ 󰌆 type                   keyword                     │
                          │ 󰈙 total_bytes            buffer identifier           │
                          │ 󰅩 .match                 postfix: match expr { ... } │
                          └────────────────────────────────────────────────────────┘
```

### Nerd Font Icons & Color Coding

| Kind | Glyph | Icon Code | Color | Purpose |
|---|:---:|:---:|---|---|
| **Method** | `󰡱` | `\u{f0871}` | Teal / Cyan | Struct / Object method |
| **Function** | `󰊕` | `\u{f0295}` | Purple / Violet | Standalone function |
| **Keyword** | `󰌆` | `\u{f0306}` | Amber / Orange | Language syntax keyword |
| **Variable** | `󰫧` | `\u{f0aef}` | Light Blue | Local variable or field |
| **Struct / Class** | `󱏒` | `\u{f13d2}` | Emerald Green | Type or struct definition |
| **Buffer Token** | `󰈙` | `\u{f0219}` | Dim Gray | Offline word from buffer |
| **Postfix Snippet**| `󰅩` | `\u{f0169}` | Rose / Red | JetBrains postfix template |

---

## 6. Trigger Rules & Keybindings

| Input | Action | Condition |
|---|---|---|
| `.`, `::`, `->` | **Auto-trigger** | Immediate popup open |
| Identifier characters | **Auto-trigger** | When word prefix length $\ge 2$ characters |
| `Ctrl + Space` | **Manual trigger** | Forces autocomplete popup open anywhere |
| `ArrowUp` / `ArrowDown` | **Navigate selection** | Moves selection index up/down |
| `Tab` or `Enter` | **Commit completion** | Cleanly replaces prefix and inserts completion |
| `Escape` | **Dismiss** | Hides popup without modifying buffer |
| Mouse Click | **Select & Insert** | Clicks any row to immediately insert |
| `Space` / `Newline` | **Dismiss** | Auto-closes when moving past current word |

---

## 7. Implementation Steps

1. **Step 1:** Create `crates/zenthree/src/completion/` module with `buffer_indexer.rs`, `keywords.rs`, `fuzzy.rs`, and `postfix.rs`.
2. **Step 2:** Implement `CompletionCoordinator` in `completion/mod.rs` to handle word prefix extraction, clean replacement, and query ranking.
3. **Step 3:** Connect the coordinator into `ui_layout/main_buffer.rs` for keystroke detection (`CharTyped`, `KeyDown`) and smart range insertion.
4. **Step 4:** Render the floating popup using `completion/ui.rs` with exact cursor positioning, scrolling, and Nerd Font icons.
5. **Step 5:** Run `cargo check -p zenthree` and validate.
