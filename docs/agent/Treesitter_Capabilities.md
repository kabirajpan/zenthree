# Tree-sitter Capabilities in ZENE Agent Architecture

## Overview

Tree-sitter is a fast, incremental parsing system that builds a concrete syntax tree (CST) for source code.
Because Zenthree already has `tree-sitter` (`crates/syntax`) embedded for Rust, Python, and JavaScript/TypeScript, the AI agent can leverage syntax-level intelligence instead of treating code as plain dumb text.

Crucially: **Tree-sitter works everywhere.**
- **Open Files (Live Buffers):** Queries the in-memory CST directly in **< 0.05 ms** (capturing unsaved user edits, 0 disk I/O).
- **Unopened Files (On-Demand):** Reads from disk and parses on the fly in **1–3 ms** at ~100,000+ lines per second.

---

## Capabilities & Tool Fit Matrix

| Area | Powered Tool / Skill | How Tree-sitter Helps | Token / Time Savings |
|------|----------------------|-----------------------|----------------------|
| **1. Code Skeleton** | `read_file(peek: true)` | Replaces bodies with `...`, leaves only types & signatures | **90% token reduction** |
| **2. Symbol Extraction** | `read_file(symbol: "...")` | Reads exact function/class boundaries (brackets/indentation) | **Zero wasted tokens** on surrounding code |
| **3. Pre-Flight Validation** | `edit_file` | Checks `has_error()` on edit in 0.5ms before saving/compiling | **Saves 5–10s** of compile turnaround |
| **4. Type Auto-Expansion** | `read_file` / `explore` | Finds structs/types used in function signature and includes them | **Fewer roundtrips** to find definitions |
| **5. Semantic Diffs** | `review` skill / `git_diff` | Identifies which functions/methods were altered, not just raw lines | **Much higher review accuracy** |
| **6. AST Chunking** | `search` / indexing | Splits code strictly on function/class boundaries, never mid-logic | **Zero broken context** |

---

## 1. Code Skeleton (File Summaries in 50 Tokens)

### The Problem
When the agent needs to understand a 1,500-line module, calling full `read_file` dumps 5,000+ tokens into the context window, cluttering attention and hitting caps.

### The Tree-sitter Solution
Tree-sitter parses the file and replaces all block bodies (`compound_statement`, `block`) with `{ ... }`, leaving imports, structs, traits, and function signatures intact.

#### Input File (`src/state/ai.rs` — 600 lines)
```rust
pub struct AIState {
    pub session_id: String,
    pub cache: Arc<RwLock<HashMap<String, Value>>>,
}

impl AIState {
    pub fn new(session_id: String) -> Self {
        // ... 20 lines of init logic ...
    }

    pub fn memory_load(&self, key: &str) -> Option<Value> {
        // ... 40 lines of cache lookup and fallback logic ...
    }

    pub fn memory_save(&mut self, key: &str, val: Value) {
        // ... 35 lines of persistence logic ...
    }
}
```

#### Skeleton Output (Generated in 1ms, ~40 tokens):
```rust
[skeleton: src/state/ai.rs (604 lines -> 18 lines)]
pub struct AIState {
    pub session_id: String,
    pub cache: Arc<RwLock<HashMap<String, Value>>>,
}

impl AIState {
    pub fn new(session_id: String) -> Self { ... }
    pub fn memory_load(&self, key: &str) -> Option<Value> { ... }
    pub fn memory_save(&mut self, key: &str, val: Value) { ... }
}
```
**Result:** The LLM immediately knows all methods and fields without wasting tokens on implementations.

---

## 2. Symbol / Function Extraction

### The Problem
Traditional text slicing requires knowing line numbers. Regex breaks on nested braces, string literals containing `{`, and decorators.

### The Tree-sitter Solution
Tree-sitter queries match the AST node by name and retrieve the exact start and end row:

#### Example: Python Function with Decorator & Multiline Signature
```python
# Query: (function_definition name: (identifier) @name (#eq? @name "verify_session")) @func

@router.post("/verify")
@limiter.limit("5/minute")
async def verify_session(
    request: Request,
    token: str = Header(...)
) -> SessionResult:
    """Verify the active user session token."""
    if not token:
        raise HTTPException(status_code=401)
    return SessionResult(valid=True, user=session.user)
```

**Tree-sitter extraction returns:**
- `start_line`: line containing `@router.post("/verify")`
- `end_line`: line containing `return SessionResult(...)`
- 100% boundary accuracy without counting spaces or braces.

---

## 3. Instant Edit Pre-Flight Validation (`edit_file`)

### The Problem
When the LLM makes an edit, it might introduce a missing bracket, syntax typo, or invalid indentation.
Waiting for `cargo check` or `tsc` takes 5 to 10 seconds.

### The Tree-sitter Solution
Before writing the edit to disk or notifying the user:
1. Apply the replacement in memory.
2. Run Tree-sitter parse on the new content:
   ```rust
   let tree = parser.parse(&new_content, None)?;
   if tree.root_node().has_error() {
       // Locate the error node in < 1ms!
       let error_node = find_syntax_error(tree.root_node());
       return Err(ToolError::ExecutionFailed(format!(
           "Edit aborted: syntax error introduced at line {}: {}",
           error_node.start_position().row + 1,
           error_node.to_sexp()
       )));
   }
   ```
3. The LLM gets immediate feedback within **1 millisecond** to fix the typo before running the compiler.

---

## 4. Semantic Context Expansion (Auto-Pulling Related Types)

### The Problem
The agent is modifying a function:
```rust
pub fn process_order(order: &OrderRequest) -> Result<OrderReceipt, PaymentError>
```
To implement it, the LLM must independently search for `OrderRequest`, `OrderReceipt`, and `PaymentError`.

### The Tree-sitter Solution
Tree-sitter extracts all type identifiers in the signature:
- Detected types: `OrderRequest`, `OrderReceipt`, `PaymentError`.
- If requested or in smart mode, the tool automatically appends their struct/enum definitions:
  ```rust
  // [Related Types in context]
  pub struct OrderRequest { pub id: String, pub items: Vec<Item>, pub total: u64 }
  pub struct OrderReceipt { pub confirmation_code: String }
  pub enum PaymentError { InsufficientFunds, ExpiredCard, GatewayTimeout }
  ```
The LLM can now write the code in a single prompt without extra exploration turns.

---

## 5. Semantic Diffs for Self-Review (`review` skill)

### The Problem
Raw line diffs (`+` and `-`) are hard for LLMs to synthesize into high-level review insights.

### The Tree-sitter Solution
Tree-sitter compares the AST before and after an edit:
- **Raw Diff:**
  ```diff
  - pub fn timeout() -> u64 { 30 }
  + pub fn timeout() -> u64 { 60 }
  ```
- **Semantic AST Diff:**
  ```
  [AST Summary]
  Modified: function `timeout` in `src/config.rs` (changed return literal 30 -> 60)
  Added:    none
  Deleted:  none
  ```
Enables the `review` skill to explain changes clearly to the user.

---

## 6. Execution Flow: Live Buffer vs Unopened File

```
                       Agent calls read_file / symbol
                                     │
                                     ▼
                   Is file currently open in editor?
                                    / \
                             YES   /   \   NO
                                  /     \
                                 v       v
                     [Live Buffer]       [On-Demand Disk Parse]
                     Read in-memory      fs::read_to_string()
                     editor buffer       Load text into parser
                           │                       │
                           ▼                       ▼
                     Active CST              Fast Tree-sitter
                     (Already in RAM)        Parse (1-3 ms)
                           │                       │
                           └───────────┬───────────┘
                                       │
                                       ▼
                       Extract Symbol / Skeleton / Validate
                                       │
                                       ▼
                         Return exact lines to LLM
```

---

## Summary

Using Tree-sitter transforms the agent from a **blind text manipulator** into a **code-aware engineer**:
1. **Never breaks function boundaries** (handles brackets, docstrings, decorators).
2. **Never burns tokens on boilerplate** (skeleton summaries).
3. **Catches syntax errors in 1ms** before slow compilers run.
4. **Works everywhere** (whether the user has the file open in an editor tab or it's untouched on disk).
