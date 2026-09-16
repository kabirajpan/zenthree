Here's a fuller architecture, in ASCII since that's what fits an IDE dev's mental model best — you can basically treat this as a module map for your Rust codebase.

## High-level layers

```
┌─────────────────────────────────────────────────────────────────┐
│                         IDE FRONTEND                             │
│   editor buffer · diff viewer · chat panel · mission tracker     │
└───────────────────────────────┬───────────────────────────────┘
                                 │ commands / events
┌───────────────────────────────▼───────────────────────────────┐
│                         ORCHESTRATOR                             │
│   owns the agent loop, task queue, retry/backoff, fallback       │
└───┬──────────────┬──────────────┬──────────────┬───────────────┘
    │              │              │              │
┌───▼───┐   ┌──────▼─────┐  ┌─────▼──────┐  ┌────▼─────┐
│CONTEXT│   │   MODEL     │  │   TOOL      │  │  STATE   │
│ENGINE │   │  PROVIDER   │  │  REGISTRY   │  │  STORE   │
│(AST,  │   │ (Claude/    │  │ (edit/run/  │  │(mission. │
│search)│   │ Gemini/…)   │  │  search)    │  │  json)   │
└───────┘   └────────────┘  └──────┬──────┘  └──────────┘
                                    │
                             ┌──────▼──────┐
                             │ SANDBOX/EXEC │
                             │ (compiler,   │
                             │  test runner)│
                             └─────────────┘
```

Every box is independently swappable. This is the difference that matters most: your Gemini 404 today should never be able to take down the whole system, because **model provider is just one pluggable box**, not something wired into the orchestrator directly.

## The orchestrator loop, in ASCII state-machine form

```
        ┌────────────┐
        │  PENDING    │
        └─────┬──────┘
              │ pick next task
        ┌─────▼──────┐
        │  PLANNING   │◄────────────────┐
        │ (ask model) │                 │
        └─────┬──────┘                 │
              │ tool_call emitted       │
        ┌─────▼──────┐                 │
        │  EXECUTING  │                 │
        └─────┬──────┘                 │
              │ tool result             │
        ┌─────▼──────┐                 │
        │ VERIFYING   │                 │
        │(compile/lint│                 │
        │ /test)      │                 │
        └─────┬──────┘                 │
       fail   │   pass                 │
   ┌──────────┴────────┐               │
   │                    │               │
┌──▼───────┐      ┌─────▼─────┐         │
│ RETRY?    │      │   DONE     │         │
│ (n<max)   │      └───────────┘         │
└──┬────┬──┘                            │
   │yes │no                             │
   │    └──────►┌───────────┐           │
   │             │  FAILED    │           │
   │             │ (log+skip) │           │
   │             └───────────┘           │
   └───────────────────────────────────┘
        (feed error back as context)
```

Notice: `FAILED` doesn't halt the orchestrator — it logs, marks that one task, and the loop keeps pulling the next task from the queue. This is the exact fix for what you're seeing in your screenshot.

## Rust-shaped design (since your IDE is Rust)

**Trait-based provider abstraction** — so Gemini going down doesn't matter:

```rust
trait ModelProvider {
    fn complete(&self, ctx: &TaskContext) -> Result<ToolCall, ProviderError>;
}

struct GeminiProvider { /* ... */ }
struct ClaudeProvider { /* ... */ }

struct ProviderChain {
    providers: Vec<Box<dyn ModelProvider>>,
}

impl ProviderChain {
    fn complete_with_fallback(&self, ctx: &TaskContext) -> Result<ToolCall, ProviderError> {
        for provider in &self.providers {
            match provider.complete(ctx) {
                Ok(call) => return Ok(call),
                Err(e) if e.is_transient() => {
                    backoff_retry(provider, ctx, 3)?; // exponential backoff
                    continue;
                }
                Err(e) => log::warn!("provider failed, falling back: {e}"),
            }
        }
        Err(ProviderError::AllProvidersFailed)
    }
}
```

**Tool registry** — a closed, typed set of actions, not free-text:

```rust
enum Tool {
    ReadFile { path: PathBuf },
    EditFile { path: PathBuf, old_str: String, new_str: String },
    RunCommand { cmd: String, args: Vec<String> },
    SearchCode { query: String },
}

trait Executor {
    fn execute(&self, tool: Tool) -> ToolResult;
}

struct ToolResult {
    ok: bool,
    output: String,
    diagnostics: Vec<Diagnostic>, // compiler/lint output goes here
}
```

**Task state as real data**, not a markdown log the model eyeballs:

```rust
#[derive(Serialize, Deserialize)]
struct Task {
    id: u32,
    description: String,
    status: TaskStatus,       // Pending, Running, Verifying, Done, Failed
    attempts: u8,
    last_error: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Mission {
    id: String,
    tasks: Vec<Task>,
}

impl Mission {
    fn next_runnable(&self) -> Option<&Task> {
        self.tasks.iter().find(|t| t.status == TaskStatus::Pending)
    }
    fn mark_failed_and_continue(&mut self, id: u32, err: String) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
            t.status = TaskStatus::Failed;
            t.last_error = Some(err);
        }
        // loop keeps going — no early return here
    }
}
```

`mission.md` can still be the *human-readable render* of this struct — generate the markdown from the struct, don't parse markdown back into state. That round-trip is fragile and is likely part of why your agent loses track after one failure.

**Verification gate** — nothing is "done" without a real check:

```rust
fn verify_task(workspace: &Path) -> VerificationResult {
    let output = Command::new("cargo")
        .args(["check", "--message-format=json"])
        .current_dir(workspace)
        .output()?;

    if output.status.success() {
        VerificationResult::Passed
    } else {
        VerificationResult::Failed(parse_diagnostics(&output.stdout))
        // these diagnostics get fed back into the NEXT model prompt
    }
}
```

## The one-sentence version of everything above

Treat the model as a **function that proposes one action given context**, never as the thing that owns control flow — your Rust orchestrator owns the loop, the retries, the fallback, and the definition of "done" (a passing compile/test), and the model just fills in one step at a time.

If you share your current orchestrator file (the one that calls Gemini and writes mission.md), I can mark up the exact lines where this state machine needs to replace what's there.