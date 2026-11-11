# Antigravity Editing and Writing Tools: Behavioral Architecture

This document details how the external **Google Antigravity** ACP provider executes file creation (**`write_file`**) and file modification (**`edit_file`**), specifically focusing on multi-area changes, protocol payloads, and diff reconciliation in the IDE engine.

---

## 1. Core Architectural Principle

Unlike traditional AI coding tools that emit fragile line-based patch commands (e.g., `replace(start_line, end_line, text)` or unified diff hunks), **Google Antigravity utilizes full-file buffer emission**.

### The Flow:
1. **Read**: The agent reads the target file into memory (`read_file`).
2. **Mutate in Memory**: The LLM applies all requested modifications (even across non-contiguous parts) in its internal neural context.
3. **Full Emission**: The agent emits the **entire updated file** in a single atomic tool call.
4. **Reconciliation**: The IDE / Client engine performs line-by-line diffing against the on-disk file to detect added/removed lines and visual hunks.

---

## 2. Tool Invocations & Payload Schemas

Antigravity uses two distinct tool names depending on whether the target file already exists or is brand new:

### A. Editing an Existing File (`edit_file`)
When modifying a file that exists on disk:
- **Tool Name**: `edit_file` (or `client_edit_file`)
- **Payload Schema**:
  ```json
  {
    "path": "/absolute/path/to/target_file.rs",
    "code_content": "<complete updated file content from line 1 to end>"
  }
  ```

### B. Creating a Brand New File (`write_file`)
When creating a file that does not yet exist:
- **Tool Name**: `write_file` (or `client_write_file` / `write_to_file`)
- **Payload Schema**:
  ```json
  {
    "path": "/absolute/path/to/target_file.rs",
    "code_content": "<complete new file content>"
  }
  ```

> [!IMPORTANT]
> **Property Key Notice:** Antigravity sends the file body in the property **`"code_content"`** (lowercase with underscore). Parsers must recognize `"code_content"` alongside `"content"` and `"CodeContent"`.

---

## 3. Multi-Area Edit Behavior (Multiple Parts in One Command)

When a user prompt instructs the agent to modify multiple separate sections in a single file at once:

> *Example Prompt:* "In `colors.txt`, change Line 03 (Green to Emerald), Line 10 (Brown to Chocolate), and Line 17 (Bronze to Platinum). Leave all other 17 lines untouched."

### Antigravity's Execution Sequence:
1. **Reads once**: `read_file` on `colors.txt`.
2. **Emits ONE single `edit_file` call**:
   - It does **not** call `edit_file` 3 times.
   - It does **not** send separate diff blocks.
   - It outputs **all 20 lines** in `"code_content"`, with lines 03, 10, and 17 updated, and lines 01–02, 04–09, 11–16, and 18–20 preserved verbatim.
3. **Verification read**: Optionally calls `read_file` to verify the written file on disk before replying to the user.

---

## 4. How the Engine Identifies and Separates the Parts

Because the payload contains the complete file rather than pre-split pieces, the **IDE Engine** is responsible for identifying where the changes occurred.

```text
       OLD FILE (on disk)                 NEW FILE (from Antigravity)
┌─────────────────────────────────┐   ┌─────────────────────────────────┐
│ Line 01: Red                    │   │ Line 01: Red                    │
│ Line 02: Blue                   │   │ Line 02: Blue                   │
│ Line 03: Green                  │   │ Line 03: Emerald                │ ◄── Area 1 (Diff)
│ Line 04: Yellow                 │   │ Line 04: Yellow                 │
│ ...                             │   │ ...                             │
│ Line 10: Brown                  │   │ Line 10: Chocolate              │ ◄── Area 2 (Diff)
│ ...                             │   │ ...                             │
│ Line 17: Bronze                 │   │ Line 17: Platinum               │ ◄── Area 3 (Diff)
└─────────────────────────────────┘   └─────────────────────────────────┘
```

### Algorithm:
1. **Load Old File**: Read the current file from disk: `let old_text = std::fs::read_to_string(path);`
2. **Receive New File**: Extract `new_text` from `args["code_content"]`.
3. **Line-by-Line Myers Diff**:
   - Compare `old_text.lines()` vs `new_text.lines()`.
   - Contiguous mismatches form individual **diff hunks**.
   - Count `lines_added` and `lines_removed`.
4. **UI Presentation**:
   - Show `Modified filename (+3, -3)` in the modified files list.
   - Render inline diff badges and trajectory cards.

---

## 5. Token Consumption vs Reliability Trade-Off

| Metric | Line/Chunk Patching | Full File Emission (Antigravity) |
| :--- | :--- | :--- |
| **Output Token Cost** | Low (only sends diff chunks) | Higher (streams the entire file) |
| **Syntax Corruption Risk** | High (models often miscount lines or lose closing braces) | Zero (full AST and brackets remain valid) |
| **Whitespace/Indent Drift** | High | Zero |
| **Patch Conflict Errors** | Frequent (causes infinite recovery loops) | Impossible (atomic file write) |
| **Generation Latency** | Marginally lower | Fast on Gemini Flash (150–250+ tokens/sec) |

### Engineering Verdict:
For files under 1,500–2,000 lines, full file emission provides **near-100% first-pass accuracy** and completely eliminates failed patch retries, justifying the higher output token usage on high-throughput models like Gemini Flash.
