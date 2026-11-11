# Tools Overview — ZENE

## How Tools Are Sent to LLM

Tools follow a **2-level lazy loading** strategy to save tokens:

```
Level 1 — Tool Manifest (sent first, always):
  name + one-line description only (~50 tokens total)

Level 2 — Full Schema (sent on demand):
  LLM requests: "give me details of search"
  → agent sends full params, types, examples for that tool only

Session Cache:
  Once LLM has seen full schema for a tool → cached for the session
  Next use: full schema sent directly, no need to request again
```

---

## Tool Manifest (Level 1 — what LLM sees first)

```
search             : Universal search — tools, skills, folders, files, symbols, or content. Combine scopes with +
read_file          : Read a file's content (by line range, symbol/function, or peek)
edit_file          : Make a surgical find-and-replace edit in a file
write_file         : Write or create a file with full content
list_directory     : List files and folders in a directory
get_project_structure : Show the directory tree of the project
get_diagnostics    : Get compiler/linter errors and warnings
run_terminal       : Execute a shell command
create_plan        : Create a step-by-step execution plan
update_plan_step   : Mark a plan step as done or in-progress
git_status         : Show changed, staged, and untracked files
git_diff           : Show the diff of current changes
delete_file        : Delete a file from disk
rename_file        : Rename or move a file
activate_skill     : Load a workflow skill by name for step-by-step guidance
```

---

## Full Tool Reference (Level 2 — on demand)

---

### `search` — Universal Discovery Tool
Search across tools, skills, and the codebase using a flexible combinator system.

**The 6 scopes** (combine any with `+`):
| Scope | What it searches | Powered by |
|-------|------------------|------------|
| `tools` | available tool names and descriptions | Tool Registry |
| `skills` | available skill names and descriptions | Skill Registry |
| `folders` | folder names in the filesystem | Filesystem |
| `files` | file names in the filesystem | Filesystem |
| `symbols` | functions, classes, structs, enums **definitions only** | **Tree-sitter AST** |
| `content` | raw text inside file content (usages, calls, strings) | Content Matcher |

```
search(
  query:  string   ← required. what to look for
  in:     string   ← optional. combine scopes with +  (default: "all" = all 6 scopes)
                     scopes: tools | skills | folders | files | symbols | content
                     e.g. "symbols", "tools+skills", "folders+files", "symbols+content"
  path:   string   ← optional. root to search from  (default: ".")
  ext:    string   ← optional. file extension filter e.g. "rs" or "rs,toml"
  regex:  bool     ← optional. treat query as regex  (default: false)
  case:   bool     ← optional. case sensitive  (default: false)
)
```

Output (compact):
```
[folders]          ← only if folder names match
  src/memory/

[files]            ← only if file names match
  src/memory/load.rs  [120 lines]

[symbols]          ← only if symbol definitions match (Tree-sitter)
  src/state/persistent.rs:14  [fn]  pub fn memory_load(key: &str) -> Option<Value>

[matches]          ← only if content matches (grouped by enclosing function)
  src/state/persistent.rs  [604 lines]  (2x)
    in fn init_cache():
      87: memory_load
      91: memory_load

1 folder  |  1 file  |  1 symbol  |  2 matches in 1 file
```

> **Rule:** Only files/folders with matches are shown. Never mention empty results.

---

### `read_file`
Read file content safely — supporting ranges, symbol extraction, batch reading, inline diagnostics, and peek mode.

```
read_file(
  path:       string   ← optional if paths is set. file path
  paths:      string[] ← optional. multiple file paths for batch reading in 1 roundtrip
  start_line: int      ← optional. first line to read (1-indexed)
  end_line:   int      ← optional. last line to read (inclusive)
  symbol:     string   ← optional. read named function(s)/struct(s) (supports "fn1,fn2")
  peek:       bool     ← optional. returns line count & outline only (0 tokens wasted)
  show_all:   bool     ← optional. reads up to 700 lines max (overriding 500 default cap)
)
```

> **Key Features & Limits:**
> - **Line caps:** Default **500 lines** cap, hard limit **700 lines**.
> - **Scope Breadcrumb:** Range reads automatically display enclosing function/class context.
> - **Inline Diagnostics:** Automatically renders active compiler errors directly beneath the offending line.
> - **Binary Shield:** Non-text files return safe metadata instead of crashing or token blowup.
> - **Batching:** Pass `paths: ["a.rs", "b.rs"]` to read multiple files in 1 roundtrip.
> - See full documentation in [Tools_ReadFile.md](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/docs/agent/Tools_ReadFile.md).

---

### `edit_file`
Make a surgical find-and-replace in a file. Minimal, precise edits only.

```
edit_file(
  path:        string   ← required. file path
  target:      string   ← required. exact text to find (must be unique in file)
  replacement: string   ← required. text to replace it with
)
```

> Prefer `edit_file` over `write_file` for any existing file. Smaller diff, safer change.

---

### `write_file`
Write or create a file with complete content.

```
write_file(
  path:    string   ← required. file path
  content: string   ← required. full content to write
)
```

> Use only for new files or when a full rewrite is necessary.

---

### `list_directory`
List files and folders in a directory (shallow).

```
list_directory(
  path:      string   ← optional. directory to list  (default: ".")
  max_items: int      ← optional. max entries to return
)
```

---

### `get_project_structure`
Show the directory tree of the project.

```
get_project_structure(
  path:      string   ← optional. root path  (default: ".")
  max_depth: int      ← optional. how deep to go  (default: 3)
)
```

> Use this to orient in an unknown project before reading specific files.

---

### `get_diagnostics`
Get compiler/linter errors and warnings from the workspace.

```
get_diagnostics()   ← no parameters
```

> Always call this after editing code to verify no new errors were introduced.

---

### `run_terminal`
Execute a shell command in the workspace.

```
run_terminal(
  command: string   ← required. shell command to run
)
```

Output includes stdout, stderr, and exit code.

> Use for: running tests, building, git commands, installing deps, checking versions.

---

### `create_plan`
Create a numbered step-by-step execution plan.

```
create_plan(
  title: string        ← optional. plan title
  steps: string[]      ← required. list of steps
)
```

---

### `update_plan_step`
Mark a plan step as completed or in progress.

```
update_plan_step(
  step_id: int     ← required. step number (1-indexed)
  status:  string  ← required. "completed" | "in_progress" | "failed"
)
```

---

### `git_status`
Show changed, staged, and untracked files.

```
git_status()   ← no parameters
```

---

### `git_diff`
Show the full diff of current changes.

```
git_diff(
  path: string   ← optional. limit diff to a specific file
)
```

---

### `delete_file`
Permanently delete a file from disk.

```
delete_file(
  path: string   ← required. file to delete
)
```

> **Destructive** — requires approval.

---

### `rename_file`
Rename or move a file.

```
rename_file(
  old_path: string   ← required. current path
  new_path: string   ← required. new path
)
```

---

### `activate_skill`
Load a workflow skill by name. Injects full step-by-step instructions into context.

```
activate_skill(
  name: string   ← required. skill name
)
```

Available built-in skills:

| Skill | When to use |
|-------|-------------|
| `explore` | Orient in an unknown codebase, locate definitions |
| `plan` | Break a complex task into phased steps |
| `implement` | Write or edit code with surgical precision |
| `debug` | Diagnose and fix errors using concrete evidence |
| `verify` | Run tests and validate changes |
| `review` | Check diffs, quality, and security before finishing |

> Skills are loaded **on demand** — only when needed. Do not activate all skills upfront.

---

## Tool Groups by Task

```
Exploring / Understanding codebase:
  search, read_file, list_directory, get_project_structure, activate_skill("explore")

Writing / Editing code:
  edit_file, write_file, read_file, get_diagnostics, activate_skill("implement")

Debugging:
  get_diagnostics, search, read_file, run_terminal, activate_skill("debug")

Running / Verifying:
  run_terminal, get_diagnostics, activate_skill("verify")

Git / Review:
  git_status, git_diff, activate_skill("review")

Planning complex tasks:
  create_plan, update_plan_step, activate_skill("plan")
```

---

## Decision Flow for LLM

```
1. Understand what user wants
2. Choose which tool group is relevant
3. If needed, activate_skill() for workflow guidance
4. Use search() to locate relevant code
5. Use read_file() only when you need full context
6. Use edit_file() for changes (write_file only for new files)
7. Use get_diagnostics() to verify no errors
8. Use run_terminal() to run tests
9. Use git_status() + git_diff() + activate_skill("review") before finishing
```

---

## Detailed Tool & Architecture Specifications

- [Tools_Search.md](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/docs/agent/Tools_Search.md) — 5-scope universal search tool design and combinator system.
- [Tools_ReadFile.md](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/docs/agent/Tools_ReadFile.md) — Safe file reading, line caps (500/700), symbol extraction, and peek mode.
- [Treesitter_Capabilities.md](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/docs/agent/Treesitter_Capabilities.md) — How Tree-sitter powers AST skeletons, instant edit validation, and live buffer sync.
