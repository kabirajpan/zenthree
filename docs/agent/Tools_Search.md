# Search Tool Design

## Overview

The `search` tool is the **universal discovery tool** for ZENE.
It searches across everything — tools, skills, and the codebase — using a flexible combinator system.

One tool. Full freedom.

---

## Tool Signature

```
search(
  query:  string   ← required. what to look for
  in:     string   ← optional. combinator of scopes  (default: "all")
  path:   string   ← optional. root path for filesystem scopes  (default: ".")
  ext:    string   ← optional. file extension filter e.g. "rs" or "rs,toml,md"
  regex:  bool     ← optional. treat query as regex  (default: false)
  case:   bool     ← optional. case sensitive  (default: false)
)
```

---

## The 6 Scopes

| Scope | What it searches | Powered by |
|-------|------------------|------------|
| `tools` | available tool names and descriptions | Tool Registry |
| `skills` | available skill names and descriptions | Skill Registry |
| `folders` | folder names in the filesystem | Filesystem |
| `files` | file names in the filesystem | Filesystem |
| `symbols` | functions, classes, structs, enums, traits **definitions only** | **Tree-sitter AST** |
| `content` | raw text inside file content (usages, calls, strings) | Content Matcher |

Combine any scopes with `+`:

```
in: "symbols"                   → only definitions (no call sites or noise)
in: "symbols+files"             → definition symbols + file names
in: "tools+skills"              → only tools and skills
in: "folders+files"             → only filesystem names
in: "symbols+content"           → definitions + all text matches
in: "all"                       → all 6 scopes (default)
```

> **Default is `"all"`** — just call `search(query: "x")` and get everything.
> Use `in: "symbols"` when looking for *where something is defined* without drowning in 50 call sites.

---

## Output Format

Compact, token-efficient. Each section only appears if it has results.

```
[tools]
  edit_file   : Make a surgical find-and-replace edit in a file
  write_file  : Write or create a file with full content

[skills]
  implement   : Surgical code edits with diff discipline
  review      : Diff review and code cleanliness check

[folders]
  src/auth/
  src/middleware/auth/

[files]
  src/auth/mod.rs  [180 lines]
  src/auth/token.rs  [95 lines]

[symbols]
  src/auth/token.rs:45  [fn]      pub fn validate_auth(token: &str) -> bool
  src/auth/types.rs:12  [struct]  AuthSession

[matches]
  src/auth/mod.rs  [180 lines]  (3x)
    in fn handle_login():
      12: pub fn validate_auth
      45: let auth_token =
    in fn refresh():
      89: auth::refresh()

  src/middleware/auth_helper.rs  [42 lines]  (1x)
    in fn verify():
      33: auth_token.verify()

2 tools  |  2 skills  |  2 folders  |  2 files  |  2 symbols  |  4 matches in 2 files
```

### Rules
- Only show sections that have results — skip empty sections silently
- `[tools]` and `[skills]` show: `name : one-line description`
- `[folders]` and `[files]` show: full path (files show `[N lines]`)
- `[symbols]` show: `filename:line  [kind]  signature` (powered by Tree-sitter)
- `[matches]` show: `filename  [N lines]  (Nx)`, grouped by enclosing function `in fn/class ...:`
- Duplicate lines counted and shown separately
- Summary line always at the bottom

---

## Usage Scenarios

### Universal — search everything (default)
```
search(query: "auth")

[tools]
  edit_file  : Make a surgical find-and-replace edit in a file

[skills]
  review     : Check diffs, quality, and security before finishing

[folders]
  src/auth/

[files]
  src/auth/mod.rs  [180 lines]

[matches]
  src/auth/mod.rs  [180 lines]  (2x)
    12: pub fn validate_auth
    45: let auth_token =

1 tool  |  1 skill  |  1 folder  |  1 file  |  2 matches in 1 file
```

---

### Discover tools only
```
search(query: "file", in: "tools")

[tools]
  read_file   : Read a file's content
  edit_file   : Make a surgical find-and-replace edit in a file
  write_file  : Write or create a file with full content
  delete_file : Delete a file from disk

4 tools
```

---

### Discover skills only
```
search(query: "debug", in: "skills")

[skills]
  debug   : Diagnose and resolve errors using concrete evidence
  verify  : Run tests and validate changes

2 skills
```

---

### Tools + Skills — discover capabilities before acting
```
search(query: "refactor", in: "tools+skills")

[skills]
  implement  : Surgical code edits with diff discipline

1 skill
```

---

### Files + Folders — locate a module
```
search(query: "auth", in: "folders+files")

[folders]
  src/auth/
  src/middleware/auth/

[files]
  src/auth/mod.rs
  src/auth/token.rs
  src/middleware/auth_helper.rs

2 folders  |  3 files
```

---

### Skills + Files — find skill and related code together
```
search(query: "test", in: "skills+files")

[skills]
  verify  : Run tests and validate changes

[files]
  src/auth/auth_test.rs
  tests/integration_test.rs

1 skill  |  2 files
```

### Symbols only — find where a function or struct is defined (Zero Noise)
```
search(query: "verify_token", in: "symbols")

[symbols]
  src/auth/token.rs:45  [fn]      pub fn verify_token(token: &str) -> Result<Claims>
  src/auth/types.rs:12  [struct]  TokenClaims

2 symbols in 2 files
```
> **Notice:** Does not return any of the 40 places where `verify_token` is called.
> Instant answer for "where is this function implemented?".

---

### Content only — who calls this function?
```
search(query: "memory_load", in: "content", ext: "rs")

[matches]
  src/state/persistent.rs (3x)
    14: fn memory_load
    87: memory_load
    91: memory_load

  src/cache/store.rs (2x)
    33: memory_load
    58: memory_load

5 matches in 2 files
```

---

### Regex — find all pub fn definitions
```
search(query: "^pub fn", in: "content", path: "src/auth", regex: true)

[matches]
  src/auth/mod.rs (4x)
    12: pub fn validate
    45: pub fn refresh_token
    89: pub fn logout
    120: pub fn get_claims

4 matches in 1 file
```

---

### Complex two-step — locate then drill in
```
# Step 1: find where the auth module lives
search(query: "auth", in: "folders+files")
  → src/auth/ found

# Step 2: drill into content inside that folder only
search(query: "validate_token", in: "content", path: "src/auth")
  → src/auth/mod.rs:45: pub fn validate_token
```

> Pattern: locate with `folders+files` → drill into `content` with `path:`
> Avoids scanning whole project when folder is already known.

---

## Why This Design

### Token efficiency
- Only shows sections that have results — no noise
- Output: name + one-liner for tools/skills, line number + keyword for matches
- LLM uses results to decide what to `read_file()` or `activate_skill()` — no content dumping

### Combinator freedom
- LLM mixes any of 5 scopes freely with `+`
- No hardcoded combinations — any of 31 possible combinations work
- LLM decides the exact scope needed for the task

### LLM does the fuzzy understanding
- LLM extracts precise keywords from vague user language before calling search
- No fuzzy matching needed — exact/regex is enough at the tool level
- Fuzzy in the tool would cause noise and confuse the LLM

---

## What This Replaces

| Old | New |
|-----|-----|
| `search_codebase(query, path, case_sensitive)` | `search(query, in:"content", path, case)` |
| `find_files(pattern, path)` | `search(query, in:"files", path)` |
| Separate tool manifest delivery | `search(query, in:"tools")` |
| Skills summary in system prompt only | `search(query, in:"skills")` |
