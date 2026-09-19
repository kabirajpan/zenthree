Agent Orchestrator Guide

A reference for building the orchestrator layer of your IDE's AI agent: what situations it must handle, and what tools it needs to do its job.
Part 1 — What the Orchestrator Actually Owns

The orchestrator is the layer between "user typed a request" and "provider + tools do the work." Its responsibilities:

    Assemble context before every model call
    Send the request to the provider
    Interpret the response (text vs. tool calls)
    Classify and route tool calls (auto-run vs. approval-gated)
    Execute tools and feed results back
    Decide when to stop looping
    Handle every failure mode without crashing or going silent
    Persist enough state to recover from interruption

Everything below is organized around these responsibilities.
Part 2 — Situations the Orchestrator Must Handle
A. Normal completion

Model responds with plain text, no tool calls. Action: append to conversation, stream to UI, stop the loop. This is the simple/happy path — most of the guide below is about everything else.
B. Single tool call

Model requests one tool. Action: classify risk → auto-run or request approval → execute → append result → loop again with the new context.
C. Multiple tool calls in one turn

Some providers return several tool calls at once (e.g. "read file A" + "read file B"). Action:

    Read-only calls: execute in parallel where possible — no reason to serialize independent reads.
    Mutating calls: execute sequentially, and re-check state between them (a second edit might depend on the outcome of the first).
    If one call in the batch fails, don't abort the others — record the failure, continue with the rest, and let the model see which succeeded and which didn't.

D. Tool execution fails

File not found, command exited non-zero, LSP not responding. Action: never crash the loop. Package the error as a tool result ({"error": "..."}) and feed it back to the model — let it adapt (try a different path, ask the user, etc.). Log it for your own debugging separately from what the model sees.
E. Model requests an unknown tool

Hallucinated tool name, or a tool that was disabled for this session. Action: don't execute anything. Return an error result naming the available tools, and let the model retry. Cap retries (e.g. 2) before surfacing to the user as a genuine failure.
F. Malformed tool arguments

JSON that doesn't match the schema, missing required fields, wrong types. Action: validate before execution, always. Return a specific error ("missing required field path") rather than a generic failure — specific errors let the model self-correct on the next turn.
G. Provider errors

    Rate limited → retry with exponential backoff; if you're multi-provider (Gemini + Groq), consider failing over to the other provider rather than just waiting.
    Network failure → retry a bounded number of times, then surface to the user plainly ("connection lost, retrying..." → "unable to reach [provider], check your connection").
    Malformed/unexpected response shape → don't crash; log the raw response for debugging, surface a generic "something went wrong" to the user.

H. Destructive action requested

run_terminal("rm -rf ..."), git push --force, delete_file on something not obviously disposable. Action: these get a stronger approval gate than ordinary mutations — surface the literal command, not just an abstract description, and consider requiring the user to type a confirmation rather than a single click for the most dangerous ones.
I. User rejects a proposed action

Action: don't just drop it — feed "user rejected this: [action]" back into the conversation as a tool result, so the model knows to try a different approach rather than repeating the same rejected suggestion.
J. User interrupts mid-execution

User cancels while the agent is mid-loop (e.g. halfway through a multi-file task). Action: stop cleanly at the next safe checkpoint — don't kill a tool execution mid-write and leave a file half-modified. Record what was completed vs. abandoned so the user can see partial progress.
K. Loop doesn't converge

Model keeps calling tools without producing a final answer. Action: hard cap on iterations (10-15 is reasonable to start). When hit, stop and tell the user plainly rather than continuing silently or crashing.
L. Context window overflow

Long conversation + large file contents exceed the model's context limit. Action: truncate or summarize older turns before hitting the limit, not after a failed call. Keep the most recent user request and tool results in full; summarize or drop older exchanges first.
M. Ambiguous or underspecified request

"Fix the bug" with no file open, or a request that could mean several different things. Action: let the model ask a clarifying question rather than guessing and taking action — this is a case where the orchestrator should allow a text-only turn even before any tools are called.
N. Streaming disconnects mid-response

Connection drops while tokens are still streaming in. Action: detect the incomplete stream, don't treat partial text as a final answer, retry the call (possibly with a note like "the previous response was cut off, please continue or retry").
O. Session resume after restart

User closes the IDE mid-task and reopens it. Action: persist conversation history and any pending approval requests to disk. On restart, offer to resume ("you had a pending edit to auth.rs — approve, reject, or discard?") rather than silently losing state or silently re-executing.
P. Concurrent requests

User fires off a new request while a previous agent loop is still running. Action: decide explicitly — queue it, cancel the in-flight one, or run them independently per-file/per-context. Don't let two loops silently interleave writes to the same file.
Part 3 — Tool Taxonomy

Organize tools into these categories. Each needs a risk level (ReadOnly, Mutating, Destructive) that drives the approval-gate logic from Part 2.
1. File operations
Tool 	Risk 	Notes
read_file(path, range?) 	ReadOnly 	Support a line range so you're not always dumping whole files
list_directory(path) 	ReadOnly 	
write_file(path, content) 	Mutating 	Full overwrite — riskier than a diff-based edit
edit_file(path, diff) 	Mutating 	Preferred over write_file — smaller, previewable, less likely to clobber unrelated changes
delete_file(path) 	Destructive 	Strong confirmation
rename_file(old, new) 	Mutating 	
2. Code understanding
Tool 	Risk 	Notes
search_codebase(query) 	ReadOnly 	Keyword or semantic
get_symbol_definition(symbol) 	ReadOnly 	Via LSP
get_references(symbol) 	ReadOnly 	Via LSP
get_file_outline(path) 	ReadOnly 	tree-sitter-based signature list, cheap context
get_diagnostics(path) 	ReadOnly 	Critical for self-correction after an edit
3. Execution
Tool 	Risk 	Notes
run_terminal(command) 	Destructive 	Broadest, riskiest tool — always gated, flag known-dangerous patterns (rm, push --force, sudo) for extra confirmation
run_tests(names?) 	Mutating→ReadOnly-ish 	Usually safe, but can have side effects (writing test artifacts); treat cautiously
get_terminal_output() 	ReadOnly 	For polling long-running commands
4. Version control
Tool 	Risk 	Notes
git_status() / git_diff() 	ReadOnly 	
git_blame(path, line) 	ReadOnly 	
git_commit(message) 	Mutating 	Gate behind approval even if "just local"
git_push() 	Destructive 	Always gated, ideally its own extra-strong confirmation
5. Project awareness
Tool 	Risk 	Notes
get_project_structure() 	ReadOnly 	High-level tree, manifests
get_open_files() 	ReadOnly 	What the user is currently looking at
get_active_selection() 	ReadOnly 	Cursor position / highlighted text
6. Planning (agent's own scratchpad, not editor-facing)
Tool 	Risk 	Notes
create_plan(steps) 	ReadOnly 	Externalizes multi-step intent before execution — show this to the user for visibility/approval on bigger tasks
update_plan_step(id, status) 	ReadOnly 	Lets the UI render live progress against the plan
Part 4 — Design Rules Worth Keeping Visible

    Read vs. write is the fundamental split. Every other rule follows from this. If you only get one thing right, get this right.
    Start with 5-6 tools, not 20. More tools = more room for the model to pick the wrong one. Add tools once the core loop is proven.
    Errors go back to the model as data, not exceptions that kill the loop. The model can often recover from "file not found" — it can't recover from a crashed process.
    Every mutating action needs a human-readable preview, not just raw arguments — a diff, not a JSON blob.
    Cap the loop. Always. No exceptions during development or in production.
    Persist state early. Retrofitting session-resume after the fact is much more painful than building it in from the start.
