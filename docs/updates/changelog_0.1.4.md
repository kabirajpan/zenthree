# Changelog v0.1.4: Generic Agent Client Protocol (ACP) Client, Google Antigravity Integration, Dynamic Model Discovery & Out-of-Process External Agents

> **Release Version:** `v0.1.4`  
> **Release Date:** September 2026  
> **Components:** `agent`, `zenthree`, `external_provider`, `acp`, `antigravity_cli`, `ui_layout`  
> **Scope:** Generic Agent Client Protocol (ACP) Client implementation using official `agent-client-protocol = "2.2.0"`, Google Antigravity external agent integration (`agy_acp_server.par`), Linux `--uid=` sandbox launch fix, zero-credential-proxy security model, client-side filesystem RPC handlers (`fs/read_text_file`, `fs/write_text_file`), permission negotiation, dynamic model discovery via `session/new`, and AI Settings Dialog External Agents management.

---

## 1. Generic Agent Client Protocol (ACP) Client Architecture

Implemented a generic, production-ready ACP client conforming to the official specification standard (used across Zed and modern AI development tooling). The architecture establishes a clean, decoupled boundary where AI agents run as separate, out-of-process subprocesses communicating strictly via JSON-RPC 2.0 over standard I/O (`stdin`/`stdout`).

### 1.1 Official Protocol Integration & Modules (`crates/agent/src/external_provider/acp/`)
* **Core Dependency:** Integrated official `agent-client-protocol = "2.2.0"` and `agent-client-protocol-derive = "2.2.0"`.
* **Subprocess Configuration ([`config.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/agent/src/external_provider/acp/config.rs)):**
  * `AcpAgentConfig`: Configurable agent registration specifying executable path, command-line arguments, environment variable overrides, working directory, and authentication methods.
  * Extensible design allowing any ACP-compliant agent (Google Antigravity, Claude Code, custom internal agents) to be configured without modifying core IDE code.
* **Protocol Data Models ([`types.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/agent/src/external_provider/acp/types.rs)):**
  * `AcpModel`: Dynamic model identifier, human-readable name, and description parsed from session responses.
  * `AcpMode`: Agent operating mode (e.g. `default`, `auto_edit`, `yolo`).
  * `AcpSlashCommand`: Slash commands exposed dynamically by the agent.
  * `AcpEvent`: Streamed lifecycle events including `Connected`, `SessionCreated`, `TextDelta`, `ThoughtDelta`, `ToolCallStarted`, `ToolCallCompleted`, `TurnCompleted`, and `Error`.
* **High-Performance Client Implementation ([`client.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/agent/src/external_provider/acp/client.rs)):**
  * `AcpClient::start(config)`: Asynchronously launches the agent subprocess with piped I/O, configures client capabilities, registers JSON-RPC request handlers, and completes the `initialize` handshake.
  * Background task execution with non-blocking broadcast channels (`tokio::sync::broadcast`) for event fanout.

---

## 2. Client-Side Filesystem & Permission Handlers

ACP is a client-delegated protocol: while the agent reasons about code and plans changes, all actual disk interactions and safety approvals are handled by the IDE client.

### 2.1 Handled Client Capabilities
* **File Reading (`fs/read_text_file`):** Agent requests text contents of any file in the workspace; `AcpClient` safely reads the file from disk and returns the content over JSON-RPC.
* **File Writing (`fs/write_text_file`):** Agent sends updated contents; `AcpClient` ensures directory parents exist and writes changes to the workspace.
* **Permission Negotiation (`session/request_permission`):** Automatic or interactive policy resolution for tool invocations and workspace mutations.

---

## 3. Google Antigravity Provider & Linux Execution Fix

Wired Google's Antigravity agent (`agy_acp_server.par`) as the flagship external provider entry in Zenthree.

### 3.1 Linux `--uid=` Launch Flag Fix
* **Issue Discovered:** Google's `agy_acp_server.par` uses Google internal launcher initialization (`init_google.cc`). When spawned on Linux systems (Ubuntu/Debian/Pop!_OS) without `--uid=`, it attempts to switch to user and group `nobody`. Because Debian/Ubuntu groups use `nogroup` rather than `nobody`, `LookupGIDByGroupName` failed with a fatal crash (`Group nobody not found`).
* **Fix Applied:** Automated `--uid=` flag insertion in `suggest_antigravity()`, preventing unnecessary privilege transitions and ensuring seamless execution across Linux distributions.

### 3.2 Dynamic Path Discovery & Zero Hardcoding
* **Automated Binary Detection:**
  * Checks `$ANTIGRAVITY_AGENT_PATH` environment variable.
  * Checks standard PATH for `agy_acp_server.par`.
  * Scans user data directories (`~/.local/share/zed/external_agents/registry/antigravity-acp/`).
* **Zero-Credential-Proxy Security Model:**
  * Antigravity manages its own Google OAuth sessions directly in `~/.gemini/antigravity-acp/acp_token.json`.
  * Zenthree **never** touches, proxies, or stores Google OAuth tokens or secrets.
  * Interactive sign-in delegates directly to `agy login` or ACP browser authentication.

---

## 4. Dynamic Model Discovery & Live Session Streaming

Eliminated hardcoded model assumptions by pulling available models, modes, and reasoning parameters dynamically directly from the agent's `session/new` response.

### 4.1 Real Discovered Models
* `gemini-3.8-flash-high`: High-reasoning ultra-fast model with dynamic thought reflection.
* `gemini-3.8-flash-medium`: Medium-reasoning model for balanced coding tasks.
* `gemini-3.8-flash-low`: Low-latency fast model for simple queries and edits.
* `gemini-3.7-flash-high`: Gemini 3.7 high-reasoning multimodal engine.
* `gemini-pro-agent`: Deep reasoning model for massive multi-file codebases and complex refactors.
* `gemini-3.1-pro-low`: Pro-tier model optimized for fast responses.

### 4.2 Stream Adapter & Provider Bridging ([`provider.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/agent/src/external_provider/antigravity_cli/provider.rs))
* `AntigravityCliProvider` wraps `AcpClient` to implement Zenthree's core `Provider` trait.
* Bridges ACP asynchronous streaming events (`AcpEvent::TextDelta`, `AcpEvent::TurnCompleted`) directly into Zenthree's synchronous `complete_stream` callback architecture.

---

## 5. UI & Settings Integration

Integrated external agent management directly into the Zenthree graphical interface and CLI binary.

### 5.1 AI & Agent Settings Dialog ([`ai_settings_dialog.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/ui_layout/ai_settings_dialog.rs))
* **External Agents Tab:**
  * Displays **Google Antigravity** connection status (`Connected` / `Not Connected`).
  * Displays active account status (`Google Account Session Active (ACP)`).
  * One-click **Sign In via Antigravity** / **Log Out** buttons.
  * Full interactive selection list of all Antigravity models with active model badge and instant activation.

### 5.2 CLI Binary Support (`zene`)
* Updated `zene` binary to test and interact with Antigravity directly from the terminal:
  * `zene --list-models`: Lists all tested Antigravity, Gemini, and Groq models.
  * `zene -p antigravity "<prompt>"`: Executes a prompt turn through ACP and streams the response.
  * `zene --login`: Launches interactive Google authentication.

---

## 6. Verification & Test Suite

* **Unit & Integration Suite:** All 70 unit tests in the `agent` crate passed cleanly.
* **Live ACP Integration Test:** Added `test_acp_client_antigravity` in `crates/agent/src/external_provider/acp/client.rs` verifying live binary spawning, initialization, session creation, dynamic model discovery, and prompt streaming.
* **Real-World Turn Execution:** Successfully verified real prompt execution and error handling against live Google server responses via `zene -p antigravity "hi"`.
