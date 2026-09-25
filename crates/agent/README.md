# ZENE Agent (`agent`)

> **Autonomous Agentic Coding Engine for Rust and Polyglot Projects**

`agent` is a high-performance, autonomous coding agent library and CLI. It powers the AI features in the **Zenthree IDE** and can also be run standalone in the terminal or integrated as an embedded crate into any Rust application.

---

## Key Features

- **⚡ Bio-Connectome Sensory Reflex Engine**: Fast intent classification (< 50 µs latency) with physical safety reflex locking against dangerous commands.
- **🛠 Level 1 Essential Toolset**:
  - `read_file`: Complete and partial file inspections with AST outline preview.
  - `edit_file`: Surgical find-and-replace code modifications preserving formatting.
  - `write_file`: Scaffolds and writes new files.
  - `list_directory`: Recursively list directory contents and file sizes.
  - `search`: Unified AST symbol, regex, and file search.
  - `run_terminal`: Native command execution in workspace working directory.
  - `get_diagnostics`: Compiler syntax and type check verification.
- **🔌 Multi-Provider Support**:
  - **Gemini** (`gemini-3.5-flash`, `gemini-3.5-flash-lite`, `gemini-2.5-flash`)
  - **Groq** (`openai/gpt-oss-120b`, `openai/gpt-oss-20b`, `qwen/qwen3.8-27b`)
- **🖥 Dual CLI Modes**:
  - **Normal Mode**: Clean, text-based terminal workflow with progress ticks.
  - **Advanced Verbose Mode**: Deep diagnostic inspection showing neuro-reflex decisions, latency in µs, formatted tool JSON parameters, and outputs.

---

## Quickstart

### 1. Configure API Keys
Create or update `.env` in this directory or your workspace root:
```env
GEMINI_API_KEY="your-gemini-key"
GROQ_API_KEY="your-groq-key"
```

### 2. Test Your Models & Quotas
Run the diagnostic script to check provider connection and latency:
```bash
./test-models.sh
```

### 3. Run in Normal Mode (Clean Text)
```bash
# Start an interactive REPL
./run-normal.sh

# Or run a single one-shot instruction
./run-normal.sh -m gemini-3.5-flash-lite "read Cargo.toml and summarize the project"
```

### 4. Run in Advanced Verbose Mode (Diagnostics)
```bash
# Start an interactive REPL with reflex latency & tool JSON tracing
./run-verbose.sh

# Or run a single one-shot instruction
./run-verbose.sh -m gemini-3.5-flash-lite "search for all struct definitions in src/"
```

---

## CLI Options & Flags

```
USAGE:
  ./run-normal.sh [OPTIONS] [PROMPT]
  ./run-verbose.sh [OPTIONS] [PROMPT]
  cargo run --bin zene-agent -- [OPTIONS] [PROMPT]

OPTIONS:
  -v, --verbose             Enable verbose diagnostic output (reflex latency, tool args & outputs)
  -p, --provider <NAME>     AI provider: 'gemini' or 'groq' (default: auto from .env)
  -m, --model <NAME>        Model identifier (e.g. gemini-3.5-flash, openai/gpt-oss-120b)
  -w, --workspace <DIR>     Target workspace directory (default: current directory)
  --list-models             List recommended models for each provider
  -h, --help                Display help message
```

---

## Interactive REPL Commands

When launched without a prompt argument, `zene-agent` enters an interactive session preserving multi-turn memory:

| Command | Action |
|---|---|
| `:help` | Show command list |
| `:verbose` | Toggle verbose mode on/off on the fly |
| `:model <name>` | Switch the active AI model mid-session |
| `:workspace <dir>` | Change active target workspace directory |
| `:clear` | Reset conversation memory |
| `:quit` or `:exit` | Exit the CLI session |

---

## Using as a Rust Library

Add to your `Cargo.toml`:
```toml
[dependencies]
agent = { path = "path/to/crates/agent" }
```

### Basic Example:
```rust
use agent::{Agent, create_agent_for_model_and_workspace};
use agent::types::event::AgentEvent;
use agent::types::approval::{ApprovalDecision, ApprovalRequest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut agent = create_agent_for_model_and_workspace(
        "gemini",
        "gemini-3.5-flash",
        "./my-project",
    ).expect("Failed to initialize agent");

    let mut on_event = |ev: AgentEvent| match ev {
        AgentEvent::ToolStarted { name, args } => println!("Calling tool: {}", name),
        AgentEvent::Text(delta) => print!("{}", delta),
        _ => {}
    };

    let mut ask_approval = |_req: ApprovalRequest| ApprovalDecision::Approve;

    let response = agent.run("Create a hello world rust project", &mut on_event, &mut ask_approval)?;
    println!("\nFinal Response:\n{}", response);
    Ok(())
}
```

---

## License
MIT or Apache-2.0.
