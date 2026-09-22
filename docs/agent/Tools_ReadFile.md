# Read File Tool Design (`read_file`)

## Overview

The `read_file` tool provides controlled, token-efficient access to file contents in the workspace.

Instead of naively dumping entire files into the agent's context window, `read_file` provides targeted reading mechanisms:
- **Range reading** (`start_line` .. `end_line`)
- **Symbol / Function reading** (`symbol: "name"`) with closing brace resolution
- **Peek mode** (`peek: true`) for zero-content metadata & outline inspection
- **Safety caps** (default 500 lines, hard cap 700 lines)

---

## Tool Signature

```
read_file(
  path:       string   ← optional if paths is provided. relative or absolute file path
  paths:      string[] ← optional. array of file paths for batch reading in 1 roundtrip
  start_line: int      ← optional. first line to read (1-indexed)
  end_line:   int      ← optional. last line to read (inclusive)
  symbol:     string   ← optional. read only a named function, struct, class, or method (supports "fn1,fn2")
  peek:       bool     ← optional. if true, return file stats & outline only (no full body)
  show_all:   bool     ← optional. read up to hard cap (700 lines) without 500-line default cap
)
```

---

## Line Limits & Industry Benchmarks

Industry standard line caps per tool call:

| Tool | Default Cap | Max / Extended Mode |
|------|-------------|---------------------|
| **GitHub Copilot Agent** | ~100 lines | 20 KiB (~400–500 lines) |
| **Cursor Agent** (Normal) | 250 lines | 750 lines (Max mode) |
| **Zed Agent** | Auto-outlining large files | Not configurable |
| **Claude Code** | 2,000 lines (chunked) | Progressive pagination |
| **ZENE `read_file`** | **500 lines** | **700 lines hard limit** |

### ZENE Rules
1. **Default Cap: 500 lines**
   - Covers >90% of real-world source files completely in a single call.
   - If a file exceeds 500 lines and no range/symbol is provided, returns first 500 lines with pagination guidance.
2. **Hard Limit: 700 lines**
   - No single `read_file` call can return more than 700 lines (even if `show_all: true` or `end_line - start_line > 700`).
   - Prevents accidental token exhaustion on 10,000+ line generated files or bundles.
   - Files larger than 700 lines require surgical reading via `start_line`..`end_line`, `symbol`, or `peek`.

---

## Reading Modes

### 1. Peek Mode (`peek: true`) — File Size & Structure Inspection

**How the LLM knows file size before reading.**
Instead of downloading hundreds or thousands of lines into the context window, `peek: true` returns immediate metadata and a symbol outline for virtually zero tokens (~15–20 tokens total).

```
read_file(path: "src/state/ai.rs", peek: true)
```

**Output:**
```
[peek: src/state/ai.rs]
  lines    : 604
  size     : 24.2 KB
  modified : 12 minutes ago
  outline  :
    14: pub struct AIState
    45: impl AIState
    48:   pub fn new() -> Self
    89:   pub fn memory_load(&self, key: &str) -> Option<Value>
    142:  pub fn memory_save(&mut self, key: &str, val: Value)
    210:  pub fn reset_context(&mut self)
    340:  fn validate_transition(&self) -> bool
```

**Why this mode is essential:**
- **Zero Token Waste:** Informs the LLM that the file has **604 lines** without burning tokens on file content.
- **Surgical Planning:** When the LLM sees `lines: 604` and `89: pub fn memory_load`, it knows to avoid a full read and immediately plans a targeted call:
  - `read_file(path: "src/state/ai.rs", symbol: "memory_load")` OR
  - `read_file(path: "src/state/ai.rs", start_line: 85, end_line: 145)`
- **Small File Confirmation:** If the file has only 40 lines, `peek` confirms that, and the LLM can safely read the full file with zero hesitation.

---

### 2. Symbol / Function Mode (`symbol: "name"`)
Reads **only** the target function, method, struct, or class, automatically finding its opening signature and closing delimiter.

```
read_file(path: "src/state/ai.rs", symbol: "memory_load")
```

**Output (Rust example):**
```
[symbol: memory_load in src/state/ai.rs (lines 89..135)]
89 |     pub fn memory_load(&self, key: &str) -> Option<Value> {
90 |         let lock = self.cache.read().ok()?;
91 |         if let Some(entry) = lock.get(key) {
...
134|         None
135|     }
```

**Output (Python example — including decorators & indentation):**
```
read_file(path: "backend/auth.py", symbol: "verify_session")

[symbol: verify_session in backend/auth.py (lines 42..68)]
42 |     @router.post("/verify")
43 |     @limiter.limit("5/minute")
44 |     async def verify_session(
45 |         request: Request,
46 |         token: str = Header(...)
47 |     ) -> SessionResult:
48 |         """Verify the active user session token."""
49 |         if not token:
50 |             raise HTTPException(status_code=401)
...
68 |         return SessionResult(valid=True, user=session.user)
```

#### Symbol Extraction Engine: Native Buffer & Tree-sitter

Zenthree already has `tree-sitter` (`crates/syntax`) for Rust, Python, and JavaScript/TypeScript. The symbol reader uses a two-tier strategy:

##### Tier 1: Live Editor Buffer + Tree-sitter AST (Preferred & Fastest)
If the requested file is currently open in a Zenthree editor buffer:
- **Zero Disk I/O:** Reads directly from the in-memory buffer (including any unsaved edits by the user!).
- **Tree-sitter CST:** Queries the active syntax tree:
  ```rust
  // Tree-sitter provides exact row/col coordinates
  let start_row = node.start_position().row + 1; // 1-indexed
  let end_row   = node.end_position().row + 1;
  ```
- **100% Grammar Accuracy:** Tree-sitter handles nested scopes, closures, async blocks, decorators, and multiline type annotations flawlessly.

##### Tier 2: Disk + Grammar Parser (or Lightweight Fallback Scanner)
If the file is not currently open in an active editor buffer:
- Parses via Tree-sitter on the read string.
- If no grammar is available for the language, falls back to the lightweight heuristic scanner:
  - **Braced Languages (C, C++, Java, Go, C#):** Tracks `{` and `}` balance counter while ignoring string literals and comments (`//`, `/* */`).
  - **Indentation Languages (Python, YAML):** Tracks leading indentation of `def`/`class` and decorators, ending at the first unindented statement.

3. **Safety Fallback**:
   - If closing delimiter cannot be determined or spans >700 lines, slices up to 700 lines and notes the truncation.

---

### 3. Sliced Range Mode with Scope Breadcrumb (`start_line` + `end_line`)
Reads an exact window of lines (1-indexed, inclusive).
Tree-sitter automatically identifies which function/class encloses the slice and renders a **breadcrumb header**:

```
read_file(path: "src/auth/token.rs", start_line: 45, end_line: 58)
```

**Output:**
```
[file: src/auth/token.rs | lines 45..58 of 180 | inside: impl TokenService -> fn verify_token()]
45 |     if token.is_empty() {
46 |         return Err(AuthError::EmptyToken);
47 |     }
48 |     let claims = self.jwt_decoder.decode(token)?;
...
58 |     Ok(claims)
```
> **Breadcrumb Advantage:** The LLM immediately understands the enclosing context without needing to read the function signature lines above.

---

### 4. Full / Default Mode with Actionable Pagination
Reads the file from line 1.

- **Files <= 500 lines**: Entire content returned.
- **Files > 500 lines**: First 500 lines returned with an **actionable, copy-pasteable pagination hint**:
  ```
  [file: src/state/ai.rs | showing lines 1..500 of 1,240]
  ...
  // [Showing 1..500 of 1,240 lines. Next chunk: read_file(path: "src/state/ai.rs", start_line: 501, end_line: 1000)]
  // [Or use peek: true to view symbol outline, or symbol: "name" to read specific functions]
  ```
- **With `show_all: true`**: Reads up to the 700-line hard cap.

---

### 5. Inline Compiler Diagnostics (Automatic Error Annotation)
When a file currently has compiler or linter errors (from `cargo check`, `tsc`, `ruff`), `read_file` renders the diagnostics **directly beneath the offending lines**:

```
read_file(path: "src/auth/session.rs", start_line: 30, end_line: 35)

[file: src/auth/session.rs | lines 30..35 of 90 | 1 compiler error]
30 | pub fn invalidate_user(user_id: &str) -> Result<()> {
31 |     let session = find_session(user_id)?;
32 |     session.purge_all_tokens();
   |             ^^^^^^^^^^^^^^^^ [rustc error: no method named `purge_all_tokens` found for struct `Session`]
33 |     Ok(())
34 | }
```
> **Why it's huge:** The LLM sees the code and the compiler error aligned side-by-side in a single tool call. Fixes happen immediately without switching tools.

---

### 6. Batch / Multi-File Reading (`paths: [...]`)
Read multiple related files in a single roundtrip:

```
read_file(paths: ["src/auth/types.rs", "src/auth/mod.rs"])
```

**Output:**
```
=== [1/2] src/auth/types.rs [45 lines] ===
pub struct Claims { pub sub: String, pub exp: u64 }
...

=== [2/2] src/auth/mod.rs [82 lines] ===
pub mod types;
pub mod token;
...
```
> **Latency Savings:** Saves 2–4 seconds of network latency by avoiding multiple sequential tool roundtrips.

---

### 7. Smart Binary & Asset Shield
Prevents crashing or token dumps if the LLM calls `read_file` on binary files:

```
read_file(path: "assets/logo.png")

[binary asset: assets/logo.png]
  type     : PNG Image (image/png)
  size     : 420 KB
  status   : Cannot display binary file as text. Use image/media tools if available.
```

---

## Interplay Between `search` and `read_file`

How the LLM avoids blind reading calls:

### Workflow A: Discovery via `search`
1. Agent runs `search(query: "memory_load", in: "symbols")`
   ```
   [symbols]
     src/state/ai.rs:89  [fn]  pub fn memory_load(&self, key: &str) -> Option<Value>
   ```
2. Agent sees the symbol and file. It calls:
   `read_file(path: "src/state/ai.rs", symbol: "memory_load")`
   ↳ **Reads only lines 89..135 (46 lines). Zero wasted tokens.**

### Workflow B: Direct File Inspection via `peek: true` (No search needed)
When the agent already knows the target file name:
1. Agent calls `read_file(path: "src/state/ai.rs", peek: true)`
2. Sees `lines: 604` and finds `89: fn memory_load` in the outline.
3. Decides to read only that slice or symbol instead of dumping 500+ lines.

### Workflow C: Multi-Symbol Reading
If two related functions need inspection together:
```
read_file(path: "src/auth/token.rs", symbol: "create_token,verify_token")
```
Returns both function blocks in one tool roundtrip.

---

## Error Handling

| Scenario | Return Value |
|----------|--------------|
| File not found | `ToolError::ExecutionFailed("File not found: ...")` |
| Path is directory | `ToolError::ExecutionFailed("Path is a directory, not a file: ...")` |
| Non-UTF8 binary file | `ToolError::ExecutionFailed("Cannot read binary file as text: ...")` |
| `symbol` not found | `ToolError::ExecutionFailed("Symbol 'xyz' not found in file: ...")` |
| `start_line > total_lines` | Empty string with note: `// [start_line exceeds total file lines]` |
| `end_line - start_line > 700` | Slices first 700 lines with hard cap notice |
