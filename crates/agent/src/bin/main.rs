use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use serde_json::Value;

use agent::types::approval::{ApprovalDecision, ApprovalRequest};
use agent::types::event::AgentEvent;
use agent::Agent;

// ANSI Colors
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const BLUE: &str = "\x1b[34m";

struct CliConfig {
    provider: String,
    model: String,
    workspace: PathBuf,
    verbose: bool,
    prompt: Option<String>,
    auto_approve: bool,
}

impl Default for CliConfig {
    fn default() -> Self {
        let has_antigravity_token =
            agent::external_provider::AntigravityAuthManager::with_default_config()
                .load_token()
                .is_some();

        let default_prov = if agent::provider::get_key("GEMINI_API_KEY").is_some() {
            "gemini"
        } else if agent::provider::get_key("GROQ_API_KEY").is_some() {
            "groq"
        } else if has_antigravity_token {
            "antigravity"
        } else {
            "gemini"
        };
        let default_mod = match default_prov {
            "antigravity" => "gemini-3.8-flash-high",
            "gemini" => "gemini-3.5-flash-lite",
            _ => "openai/gpt-oss-120b",
        };

        Self {
            provider: default_prov.to_string(),
            model: default_mod.to_string(),
            workspace: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            verbose: false,
            prompt: None,
            auto_approve: false,
        }
    }
}

fn print_version() {
    println!("zene {}", env!("CARGO_PKG_VERSION"));
}

fn print_missing_keys_error() {
    eprintln!("\n{RED}{BOLD}✗ Error: No AI provider credentials found.{RESET}");
    eprintln!("\nZENE can operate via Cloud APIs, Google Account, or 100% Offline Local Models:\n");
    eprintln!("  1. {CYAN}{BOLD}Local Offline Models{RESET} (Zero API Keys, Private, On-Device):");
    eprintln!("     View catalog:  {GREEN}zene --local-models{RESET}");
    eprintln!("     Download:      {GREEN}zene --download-model qwen2.5-coder-1.5b-q4{RESET}");
    eprintln!("     Run directly:  {GREEN}zene -p local -m qwen2.5-coder-1.5b-q4{RESET}\n");
    eprintln!("  2. {CYAN}{BOLD}Google Antigravity{RESET} (Use your own Google ID via OAuth):");
    eprintln!("     Login directly: {GREEN}zene --login{RESET} or run with {GREEN}zene -p antigravity{RESET}\n");
    eprintln!("  3. {CYAN}{BOLD}Google Gemini{RESET} (API Key):");
    eprintln!("     Get key: {DIM}https://aistudio.google.com/app/apikey{RESET}");
    eprintln!("     Set it:  {GREEN}export GEMINI_API_KEY=\"your-key-here\"{RESET}\n");
    eprintln!("  4. {CYAN}{BOLD}Groq{RESET} (Ultra-low latency inference):");
    eprintln!("     Get key: {DIM}https://console.groq.com/keys{RESET}");
    eprintln!("     Set it:  {GREEN}export GROQ_API_KEY=\"your-key-here\"{RESET}\n");
}

fn print_help() {
    println!("{BOLD}{CYAN}ZENE Agent CLI{RESET} — Autonomous Agentic Coding Engine");
    println!("Standalone CLI & IDE backend for autonomous codebase modification\n");
    println!("{BOLD}USAGE:{RESET}");
    println!("  zene [OPTIONS] [PROMPT]");
    println!("  cargo run --bin zene -- [OPTIONS] [PROMPT]");
    println!("  ./run-normal.sh [OPTIONS] [PROMPT]\n");
    println!("{BOLD}OPTIONS:{RESET}");
    println!("  {GREEN}-v, --verbose{RESET}             Enable verbose diagnostic output (reflex latency, full tool args & outputs)");
    println!(
        "  {GREEN}-p, --provider <NAME>{RESET}     AI provider: 'local', 'antigravity', 'gemini', or 'groq'"
    );
    println!("  {GREEN}-m, --model <NAME>{RESET}        Model identifier (e.g. qwen2.5-coder-1.5b-q4, gemini-3.8-flash-high)");
    println!("  {GREEN}-w, --workspace <DIR>{RESET}     Target workspace directory (default: current directory)");
    println!("  {GREEN}--login{RESET}                   Log in with your Google Account / Google ID for Antigravity");
    println!("  {GREEN}--list-models{RESET}             List recommended models for each provider");
    println!("  {GREEN}--local-models{RESET}            List installed & downloadable local GGUF models");
    println!("  {GREEN}--download-model <ID>{RESET}     Download a local model with interactive progress");
    println!("  {GREEN}-V, --version{RESET}             Print version information");
    println!("  {GREEN}-h, --help{RESET}                Display this help message\n");
    println!("{BOLD}INTERACTIVE COMMANDS:{RESET} (when launched without [PROMPT])");
    println!("  {YELLOW}:help{RESET}                     Show command list");
    println!("  {YELLOW}:models{RESET}                   List all models (cloud & local)");
    println!("  {YELLOW}:local-models{RESET}             List local models and download status");
    println!("  {YELLOW}:download <ID>{RESET}            Download and register a local model");
    println!("  {YELLOW}:rules{RESET}                    Show active project rules (AGENTS.md, .agents/rules)");
    println!("  {YELLOW}:trust [on|off]{RESET}           View or set workspace trust status");
    println!("  {YELLOW}:approve{RESET}                  Toggle manual/auto-approve mode on/off");
    println!("  {YELLOW}:login{RESET}                    Log in with your Google Account");
    println!("  {YELLOW}:verbose{RESET}                  Toggle verbose diagnostic mode on/off");
    println!("  {YELLOW}:model <NAME>{RESET}             Switch model on the fly");
    println!("  {YELLOW}:clear{RESET}                    Reset conversation memory");
    println!("  {YELLOW}:quit / :exit{RESET}             Exit the interactive session\n");
    println!("{BOLD}SLASH COMMANDS:{RESET} (Autonomous directives & quick actions)");
    println!(
        "  {CYAN}/goal <prompt>{RESET}            Run long-running task until thoroughly achieved"
    );
    println!("  {CYAN}/plan <prompt>{RESET}            Formulate structured step-by-step milestones before execution");
    println!("  {CYAN}/fix <prompt>{RESET}             Targeted diagnostics and minimal repair");
    println!("  {CYAN}/grill-me <topic>{RESET}         Interactive interview to clarify ambiguous requirements");
    println!("  {CYAN}/compact{RESET}                  Compact prior context & tool outputs");
    println!("  {CYAN}/tasks{RESET}                    List background processes and daemons");
    println!("  {CYAN}/schedule <prompt>{RESET}        Schedule timers or recurring triggers");
    println!(
        "  {CYAN}/learn <rule>{RESET}             Persist learned knowledge/rules to workspace\n"
    );
}

fn print_local_models() {
    let installed = agent::local_model::LocalModelManager::list_installed();
    let models_dir = agent::local_model::LocalModelManager::models_dir();

    println!("{BOLD}{CYAN}═══════════════════════════════════════════════════════════════{RESET}");
    println!("{BOLD}{CYAN}                 ZENE Local Model Management                   {RESET}");
    println!("{BOLD}{CYAN}═══════════════════════════════════════════════════════════════{RESET}");
    println!("  {BOLD}Storage Path:{RESET} {DIM}{}{RESET}\n", models_dir.display());

    println!("{BOLD}Installed Local Models ({}):{RESET}", installed.len());
    if installed.is_empty() {
        println!("  {DIM}No local models downloaded yet.{RESET}");
    } else {
        for m in &installed {
            println!(
                "  • {GREEN}{BOLD}{:<32}{RESET} [{YELLOW}{}{RESET}] {DIM}{}{RESET}",
                m.id, m.file_size_display, m.filename
            );
        }
    }

    println!("\n{BOLD}Available Catalog Models for Download:{RESET}");
    for m in agent::local_model::MODEL_CATALOG {
        let is_inst = agent::local_model::LocalModelManager::is_downloaded(&m.id);
        let status = if is_inst {
            format!("{GREEN}✓ INSTALLED{RESET}")
        } else {
            format!("{CYAN}[DOWNLOADABLE]{RESET}")
        };
        println!(
            "  • {BOLD}{:<32}{RESET} {} ({}) - {}",
            m.id, status, m.size_display, m.name
        );
        println!("    {DIM}Purpose: {} | Ollama: {}{RESET}", m.recommended_for, m.ollama_tag);
    }

    println!("\n{BOLD}Quick Commands:{RESET}");
    println!("  • Download: {GREEN}zene --download-model <id>{RESET} (e.g. {CYAN}zene --download-model qwen2.5-coder-1.5b-q4{RESET})");
    println!("  • Run agent: {GREEN}zene -p local -m <id>{RESET} (e.g. {CYAN}zene -p local -m qwen2.5-coder-1.5b-q4{RESET})\n");
}

fn download_model_cli(id_or_tag: &str) -> Result<(), i32> {
    let cat = match agent::local_model::find_catalog_model(id_or_tag) {
        Some(c) => c,
        None => {
            eprintln!("{RED}Model '{}' not found in catalog.{RESET}", id_or_tag);
            eprintln!("Run {GREEN}zene --local-models{RESET} to view all available models.");
            return Err(1);
        }
    };

    println!("{BOLD}{CYAN}Starting Download:{RESET} {GREEN}{}{RESET} ({})", cat.name, cat.size_display);
    println!("  URL: {DIM}{}{RESET}", cat.download_url);
    println!("  Target: {DIM}{}{RESET}\n", agent::local_model::LocalModelManager::models_dir().join(&cat.filename).display());

    let mut last_pct = -1;
    let res = agent::local_model::ModelDownloader::download(cat, None, |p| {
        let pct_int = p.percentage as i32;
        if pct_int != last_pct || p.is_finished {
            last_pct = pct_int;
            let bar_len: usize = 30;
            let filled = ((p.percentage / 100.0) * bar_len as f32) as usize;
            let empty = bar_len.saturating_sub(filled);
            let bar = format!("{}{}", "=".repeat(filled), " ".repeat(empty));
            let speed_mb = p.speed_bytes_per_sec as f64 / (1024.0 * 1024.0);
            let downloaded_mb = p.downloaded_bytes as f64 / (1024.0 * 1024.0);
            let total_mb = p.total_bytes as f64 / (1024.0 * 1024.0);
            let eta = p.eta_seconds.map(|s| format!("{}s", s)).unwrap_or_else(|| "--".to_string());

            print!("\r  [{CYAN}{bar}{RESET}] {BOLD}{:5.1}%{RESET} ({:.1}MB / {:.1}MB) {YELLOW}{:.2} MB/s{RESET} ETA: {eta}   ",
                p.percentage, downloaded_mb, total_mb, speed_mb);
            let _ = std::io::stdout().flush();
        }
    });

    println!();
    match res {
        Ok(path) => {
            println!("\n{BOLD}{GREEN}✓ Successfully downloaded and installed model!{RESET}");
            println!("  Path: {DIM}{}{RESET}", path.display());
            println!("  Run with: {BOLD}zene -p local -m {}{RESET}\n", cat.id);
            Ok(())
        }
        Err(e) => {
            eprintln!("\n{BOLD}{RED}✗ Download failed:{RESET} {e}");
            Err(1)
        }
    }
}

fn print_models() {
    println!("{BOLD}{CYAN}Available & Tested Models:{RESET}\n");
    println!("{BOLD}Local Offline Models (Zero API Keys, Private, On-Device):{RESET}");
    let installed = agent::local_model::LocalModelManager::list_installed();
    if installed.is_empty() {
        println!("  {DIM}(No local models downloaded yet. Run 'zene --local-models' or 'zene --download-model <id>'){RESET}");
    } else {
        for m in &installed {
            println!("  • {GREEN}{:<28}{RESET} [{}] (Local file)", m.id, m.file_size_display);
        }
    }
    for m in agent::local_model::MODEL_CATALOG {
        if !agent::local_model::LocalModelManager::is_downloaded(&m.id) {
            println!("  • {DIM}{:<28} [{}] (Download via: zene --download-model {}){RESET}", m.id, m.size_display, m.id);
        }
    }

    println!("\n{BOLD}Google Antigravity Models (Authenticate with your Google ID):{RESET}");
    for m in agent::external_provider::available_models() {
        println!("  • {GREEN}{:<28}{RESET} {}", m.id, m.description);
    }
    println!("\n{BOLD}Gemini API Models (Via GEMINI_API_KEY):{RESET}");
    println!(
        "  • {GREEN}gemini-3.5-flash-lite{RESET}       Ultra-lightweight, high throughput (Default)"
    );
    println!("  • {GREEN}gemini-3.5-flash{RESET}            Fast, full function-calling, generous daily quota\n");
    println!("{BOLD}Groq Models (Via GROQ_API_KEY):{RESET}");
    println!("  • {GREEN}openai/gpt-oss-120b{RESET}         High reasoning, full tool calling");
    println!("  • {GREEN}openai/gpt-oss-20b{RESET}          Fast lightweight reasoning\n");
}

fn parse_args() -> Result<CliConfig, i32> {
    let mut config = CliConfig::default();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut positional = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return Err(0);
            }
            "-V" | "--version" => {
                print_version();
                return Err(0);
            }
            "--list-models" => {
                print_models();
                return Err(0);
            }
            "--local-models" | "--list-local-models" => {
                print_local_models();
                return Err(0);
            }
            "--download-model" | "--download" => {
                if i + 1 < args.len() {
                    let model_to_download = args[i + 1].clone();
                    download_model_cli(&model_to_download)?;
                    return Err(0);
                } else {
                    eprintln!("{RED}Error: --download-model requires a model identifier (e.g. qwen2.5-coder-1.5b-q4){RESET}");
                    return Err(1);
                }
            }
            "--login" | "login" => {
                println!("{BOLD}{CYAN}Initiating Google Antigravity OAuth Login...{RESET}");
                let auth = agent::external_provider::AntigravityAuthManager::with_default_config();
                match auth.login_interactive() {
                    Ok(()) => {
                        println!(
                            "\n{BOLD}{GREEN}✓ Successfully authenticated via Antigravity!{RESET}"
                        );
                        println!(
                            "  Session saved to: {DIM}{}{RESET}",
                            agent::external_provider::antigravity_cli::auth::default_token_path()
                                .display()
                        );
                        println!("  You can now run: {BOLD}zene -p antigravity{RESET}\n");
                        return Err(0);
                    }
                    Err(e) => {
                        eprintln!("\n{BOLD}{RED}✗ Login failed:{RESET} {e}");
                        return Err(1);
                    }
                }
            }
            "-v" | "--verbose" => {
                config.verbose = true;
            }
            "-y" | "--yes" | "--auto-approve" => {
                config.auto_approve = true;
            }
            "-p" | "--provider" => {
                if i + 1 < args.len() {
                    config.provider = args[i + 1].to_lowercase();
                    if config.provider == "antigravity" && config.model == "gemini-3.5-flash-lite" {
                        config.model = "gemini-3.8-flash-high".to_string();
                    } else if config.provider == "local" && (config.model.starts_with("gemini") || config.model.starts_with("openai")) {
                        let installed = agent::local_model::LocalModelManager::list_installed();
                        if let Some(first) = installed.first() {
                            config.model = first.id.clone();
                        } else {
                            config.model = "qwen2.5-coder-1.5b-q4".to_string();
                        }
                    }
                    i += 1;
                }
            }
            "-m" | "--model" => {
                if i + 1 < args.len() {
                    config.model = args[i + 1].clone();
                    if config.provider != "antigravity" && config.provider != "gemini" && config.provider != "groq" {
                        if agent::local_model::LocalModelManager::is_downloaded(&config.model)
                            || agent::local_model::find_catalog_model(&config.model).is_some()
                        {
                            config.provider = "local".to_string();
                        }
                    }
                    i += 1;
                }
            }
            "-w" | "--workspace" => {
                if i + 1 < args.len() {
                    config.workspace = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            other => {
                if other.starts_with('-') {
                    eprintln!("{RED}Unknown option: {}{RESET}", other);
                    print_help();
                    return Err(1);
                }
                positional.push(other.to_string());
            }
        }
        i += 1;
    }

    if !positional.is_empty() {
        config.prompt = Some(positional.join(" "));
    }

    Ok(config)
}

fn format_tool_args_summary(args: &Value) -> String {
    if let Some(obj) = args.as_object() {
        let parts: Vec<String> = obj
            .iter()
            .take(3)
            .map(|(k, v)| {
                let v_str = match v {
                    Value::String(s) => {
                        if s.len() > 40 {
                            format!("\"{}...\"", &s[..37])
                        } else {
                            format!("\"{}\"", s)
                        }
                    }
                    other => other.to_string(),
                };
                format!("{}: {}", k, v_str)
            })
            .collect();
        parts.join(", ")
    } else {
        String::new()
    }
}

fn execute_turn(
    agent: &mut Agent,
    prompt: &str,
    verbose: bool,
    auto_approve: bool,
) -> Result<String, String> {
    if verbose {
        println!("{DIM}─────────────────────────────────────────────────────────────{RESET}");
        println!("{BOLD}{CYAN}⚡ Bio-Connectome Sensory Reflex Engine:{RESET}");
        if let Some(decision) = agent.reflex().classify(prompt) {
            let intent_color = if decision.is_danger { RED } else { GREEN };
            println!(
                "  • Intent: {intent_color}{:?}{RESET} | Reaction Time: {BOLD}{:.2} µs{RESET} | Confidence: {:.1}%",
                decision.intent, decision.reaction_time_us, decision.confidence
            );
            println!("  • Explanation: {DIM}{}{RESET}", decision.explanation);
            if decision.is_danger {
                println!("  {RED}{BOLD}🚨 PHYSICAL SAFETY REFLEX LOCK ENGAGED! Workspace mutation blocked.{RESET}");
            }
        }
        println!("{DIM}─────────────────────────────────────────────────────────────{RESET}");
    }

    let mut stream_started = false;

    let mut on_event = |ev: AgentEvent| match ev {
        AgentEvent::ToolStarted { name, args } => {
            if verbose {
                println!("\n{BOLD}{YELLOW}⚙ [TOOL CALL]{RESET} {BOLD}{}{RESET}", name);
                if let Ok(pretty) = serde_json::to_string_pretty(&args) {
                    for line in pretty.lines() {
                        println!("  {DIM}{}{RESET}", line);
                    }
                }
            } else {
                let summary = format_tool_args_summary(&args);
                println!("  {YELLOW}⚙ {name}{RESET} {DIM}({}){RESET}", summary);
            }
        }
        AgentEvent::ToolFinished { name, result } => {
            let res_str = match &result {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            if verbose {
                let snippet = if res_str.len() > 600 {
                    format!(
                        "{}...\n[truncated {} bytes]",
                        &res_str[..600],
                        res_str.len() - 600
                    )
                } else {
                    res_str
                };
                println!("{BOLD}{GREEN}✓ [TOOL RESULT]{RESET} {BOLD}{}{RESET}", name);
                for line in snippet.lines() {
                    println!("  {DIM}{}{RESET}", line);
                }
                println!();
            } else {
                let lines = res_str.lines().count();
                println!(
                    "  {GREEN}✓ {name}{RESET} {DIM}finished ({} lines output){RESET}",
                    lines
                );
            }
        }

        AgentEvent::Thought(thought) => {
            if verbose {
                print!("{DIM}{thought}{RESET}");
                let _ = io::stdout().flush();
            }
        }
        AgentEvent::Text(delta) => {
            if !stream_started {
                if !verbose {
                    println!();
                }
                stream_started = true;
            }
            print!("{}", delta);
            let _ = io::stdout().flush();
        }
        AgentEvent::Failed(err) => {
            eprintln!("\n{RED}{BOLD}✗ Error:{RESET} {err}");
        }
        AgentEvent::Finished => {}
        _ => {}
    };

    let mut ask_approval = |req: ApprovalRequest| {
        if auto_approve {
            return ApprovalDecision::Approve;
        }
        println!("\n{YELLOW}{BOLD}⚠️ APPROVAL REQUIRED FOR TOOL ACTION:{RESET}");
        println!("  {DIM}{}{RESET}", req.preview);
        print!("{BOLD}Proceed with execution? [y/N]: {RESET}");
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let trimmed = input.trim().to_lowercase();
            if trimmed == "y" || trimmed == "yes" {
                ApprovalDecision::Approve
            } else {
                println!("{RED}Action rejected by user.{RESET}");
                ApprovalDecision::Reject
            }
        } else {
            ApprovalDecision::Reject
        }
    };

    let res = agent.run(prompt, &mut on_event, &mut ask_approval);

    if stream_started {
        println!();
    }

    res.map_err(|e| format!("{:?}", e))
}

fn create_configured_agent(config: &CliConfig) -> Option<Agent> {
    let mut agent = agent::create_agent_for_model_and_workspace(
        &config.provider,
        &config.model,
        &config.workspace,
    )?;

    let is_local = config.provider == "local";
    let mut sys_msg = if is_local {
        format!(
            "You are an autonomous AI coding assistant operating in workspace: {}\n\n\
            Guidelines:\n\
            - DIRECT ACTION & IMMEDIATE EXECUTION: When the user asks you to create, scaffold, or implement a file, program, or game, take immediate action using `write_file`. Do NOT call `create_plan`. Call `write_file` directly with the complete code content.\n\
            - Do NOT refuse. Do NOT say 'I cannot write code'. Call `write_file` directly with the complete file content.\n\
            - You have full access to tools: `write_file(path, content)`, `read_file(path)`, `list_directory(path)`, and `run_terminal(command)`.",
            config.workspace.display()
        )
    } else {
        format!(
            "You are a helpful, accurate, and concise AI coding assistant.\n\
            Workspace: {}\n\n\
            Guidelines:\n\
            - FOCUS ON THE USER'S TASK: Focus strictly on answering the user's questions about their project, codebase, and code. Do NOT talk about yourself, your underlying model/platform, or the IDE unless the user explicitly asks about it.\n\
            - CASUAL CHAT & GREETINGS: When the user simply says 'hi', 'hello', or greets you, reply warmly, concisely, and conversationally. Do NOT call file reading or directory listing tools unless the user specifically asks a question about the workspace or requests a task.\n\
            - DIRECT ACTION & IMMEDIATE EXECUTION: When the user asks you to create, scaffold, implement, fix, or run something, take immediate action using your available tools. Do NOT pause execution or create unnecessary markdown planning files for straightforward user tasks. Execute commands and edits directly.\n\
            - STRICT PROJECT BOUNDARY: You operate inside the workspace project root. Do not navigate to parent directories.\n\
            - Use `search`, `read_file`, and `list_directory` to explore files, symbols, and directory trees.\n\
            - Run `get_diagnostics` or test/build suites via `run_terminal` to verify your changes.",
            config.workspace.display()
        )
    };

    if !is_local {
        let rules_block = agent.rules_prompt();
        if !rules_block.is_empty() {
            sys_msg.push_str("\n\n");
            sys_msg.push_str(&rules_block);
        }

        sys_msg.push_str("\n\n");
        sys_msg.push_str(&agent::orchestrator::format_slash_commands_prompt());
    }

    agent.push_system_message(sys_msg);
    Some(agent)
}

fn run_interactive(mut config: CliConfig) {
    println!("{BOLD}{CYAN}═══════════════════════════════════════════════════════════════{RESET}");
    println!("{BOLD}{CYAN}           ZENE Agent Interactive Developer CLI                {RESET}");
    println!("{BOLD}{CYAN}═══════════════════════════════════════════════════════════════{RESET}");
    println!(
        "  {BOLD}Provider:{RESET}   {GREEN}{}{RESET}",
        config.provider
    );
    println!("  {BOLD}Model:{RESET}      {GREEN}{}{RESET}", config.model);
    println!(
        "  {BOLD}Workspace:{RESET}  {BLUE}{}{RESET}",
        config.workspace.display()
    );
    println!(
        "  {BOLD}Verbose:{RESET}    {}",
        if config.verbose {
            format!("{YELLOW}ON{RESET}")
        } else {
            format!("{DIM}OFF{RESET}")
        }
    );
    println!(
        "  {BOLD}Approval:{RESET}   {}",
        if config.auto_approve {
            format!("{YELLOW}AUTO-APPROVE{RESET}")
        } else {
            format!("{GREEN}MANUAL (Prompt on destructive tools){RESET}")
        }
    );

    let agent_opt = create_configured_agent(&config);

    let mut agent = match agent_opt {
        Some(a) => a,
        None => {
            print_missing_keys_error();
            return;
        }
    };

    let trust_display = match agent.trust_status() {
        agent::trust::TrustStatus::Trusted => format!("{GREEN}Trusted (Full Access){RESET}"),
        agent::trust::TrustStatus::Untrusted => format!(
            "{RED}Untrusted (Restricted Sandbox: Shell blocked, edits require approval){RESET}"
        ),
        agent::trust::TrustStatus::Unknown => format!("{YELLOW}Unknown / Unverified{RESET}"),
    };
    println!("  {BOLD}Trust:{RESET}       {}", trust_display);

    let rules_count = agent.rules().len();
    if rules_count > 0 {
        println!(
            "  {BOLD}Rules:{RESET}       {GREEN}{} active project rule(s) loaded{RESET}",
            rules_count
        );
    }
    println!("{DIM}Type your instructions or ':help' for commands. ':quit' to exit.{RESET}\n");

    let stdin = io::stdin();
    let mut session_json: Option<String> = None;

    loop {
        print!("{BOLD}{CYAN}zene>{RESET} ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
            break; // EOF
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with(':') {
            match trimmed {
                ":q" | ":quit" | ":exit" => {
                    println!("{DIM}Exiting ZENE CLI. Goodbye!{RESET}");
                    break;
                }
                ":help" => {
                    print_help();
                    continue;
                }
                ":approve" | ":auto-approve" => {
                    config.auto_approve = !config.auto_approve;
                    println!(
                        "Auto-approve mode: {}",
                        if config.auto_approve {
                            format!("{YELLOW}ON (Destructive actions auto-run){RESET}")
                        } else {
                            format!("{GREEN}OFF (Prompting for review){RESET}")
                        }
                    );
                    continue;
                }
                ":rules" => {
                    let rules = agent.rules();
                    if rules.is_empty() {
                        println!("{DIM}No project rules found. Add 'AGENTS.md', 'GEMINI.md', or '.agents/rules/*.md' in your workspace.{RESET}");
                    } else {
                        println!(
                            "{BOLD}{CYAN}Active Discovered Rules ({}):{RESET}",
                            rules.len()
                        );
                        for r in rules {
                            println!(
                                "  • {GREEN}{}{RESET} ({:?}) -> {DIM}{}{RESET}",
                                r.name,
                                r.scope,
                                r.path.display()
                            );
                        }
                    }
                    continue;
                }
                cmd if cmd == ":trust" || cmd.starts_with(":trust ") => {
                    let sub = cmd.strip_prefix(":trust").unwrap_or("").trim();
                    let mgr = agent::trust::WorkspaceTrustManager::default();
                    if sub == "on" || sub == "trust" || sub == "yes" || sub == "true" {
                        if let Err(e) =
                            mgr.set_trust(&config.workspace, agent::trust::TrustStatus::Trusted)
                        {
                            eprintln!("{RED}Failed to update trust: {e}{RESET}");
                        } else {
                            agent.set_trust_status(agent::trust::TrustStatus::Trusted);
                            println!("{GREEN}✓ Workspace marked as TRUSTED. Full shell and tool execution permitted.{RESET}");
                        }
                    } else if sub == "off" || sub == "untrust" || sub == "no" || sub == "false" {
                        if let Err(e) =
                            mgr.set_trust(&config.workspace, agent::trust::TrustStatus::Untrusted)
                        {
                            eprintln!("{RED}Failed to update trust: {e}{RESET}");
                        } else {
                            agent.set_trust_status(agent::trust::TrustStatus::Untrusted);
                            println!("{YELLOW}⚠️ Workspace marked as UNTRUSTED. Terminal commands are blocked; file writes require approval.{RESET}");
                        }
                    } else if sub.is_empty() {
                        let status_str = match agent.trust_status() {
                            agent::trust::TrustStatus::Trusted => format!("{GREEN}Trusted{RESET}"),
                            agent::trust::TrustStatus::Untrusted => {
                                format!("{RED}Untrusted (Restricted Sandbox){RESET}")
                            }
                            agent::trust::TrustStatus::Unknown => {
                                format!("{YELLOW}Unknown / Unverified{RESET}")
                            }
                        };
                        println!("Workspace: {}", config.workspace.display());
                        println!("Trust status: {}", status_str);
                        println!(
                            "{DIM}Use ':trust on' or ':trust off' to change trust status.{RESET}"
                        );
                    } else {
                        println!("Usage: :trust [on|off]");
                    }
                    continue;
                }
                ":login" => {
                    println!("{BOLD}{CYAN}Initiating Google Antigravity OAuth Login...{RESET}");
                    let auth =
                        agent::external_provider::AntigravityAuthManager::with_default_config();
                    match auth.login_interactive() {
                        Ok(_) => {
                            println!("{GREEN}✓ Successfully authenticated with Google Antigravity!{RESET}");
                            config.provider = "antigravity".to_string();
                            config.model = "gemini-3.8-flash-high".to_string();
                            if let Some(fresh) = agent::create_agent_for_model_and_workspace(
                                &config.provider,
                                &config.model,
                                &config.workspace,
                            ) {
                                agent = fresh;
                                println!("{GREEN}Switched active session to Google Antigravity (Gemini 3.8 Flash).{RESET}");
                            }
                        }
                        Err(e) => {
                            eprintln!("{RED}Login failed: {e}{RESET}");
                        }
                    }
                    continue;
                }
                ":verbose" => {
                    config.verbose = !config.verbose;
                    println!(
                        "Verbose mode: {}",
                        if config.verbose {
                            format!("{YELLOW}ON{RESET}")
                        } else {
                            format!("{DIM}OFF{RESET}")
                        }
                    );
                    continue;
                }
                ":models" => {
                    print_models();
                    continue;
                }
                ":local-models" => {
                    print_local_models();
                    continue;
                }
                cmd if cmd.starts_with(":download ") => {
                    let id_to_download = cmd.strip_prefix(":download ").unwrap().trim();
                    if !id_to_download.is_empty() {
                        let _ = download_model_cli(id_to_download);
                    } else {
                        println!("{RED}Usage: :download <model-id> (e.g. :download qwen2.5-coder-1.5b-q4){RESET}");
                    }
                    continue;
                }
                ":clear" => {
                    session_json = None;
                    if let Some(fresh) = agent::create_agent_for_model_and_workspace(
                        &config.provider,
                        &config.model,
                        &config.workspace,
                    ) {
                        agent = fresh;
                        println!("{GREEN}Conversation context cleared.{RESET}");
                    }
                    continue;
                }
                cmd if cmd.starts_with(":model ") => {
                    let new_model = cmd.strip_prefix(":model ").unwrap().trim();
                    if !new_model.is_empty() {
                        config.model = new_model.to_string();
                        if agent::local_model::LocalModelManager::is_downloaded(&config.model)
                            || agent::local_model::find_catalog_model(&config.model).is_some()
                        {
                            config.provider = "local".to_string();
                        }
                        if let Some(mut fresh) = agent::create_agent_for_model_and_workspace(
                            &config.provider,
                            &config.model,
                            &config.workspace,
                        ) {
                            if let Some(ref json) = session_json {
                                let _ = fresh.import_session_json(json);
                            }
                            agent = fresh;
                            println!("{GREEN}Switched model to: {} (provider: {}){RESET}", config.model, config.provider);
                        } else {
                            println!("{RED}Failed to switch to model: {}{RESET}", config.model);
                        }
                    }
                    continue;
                }
                cmd if cmd.starts_with(":workspace ") => {
                    let new_ws = cmd.strip_prefix(":workspace ").unwrap().trim();
                    config.workspace = PathBuf::from(new_ws);
                    if let Some(fresh) = agent::create_agent_for_model_and_workspace(
                        &config.provider,
                        &config.model,
                        &config.workspace,
                    ) {
                        agent = fresh;
                        println!(
                            "{GREEN}Switched workspace to: {}{RESET}",
                            config.workspace.display()
                        );
                    }
                    continue;
                }
                _ => {
                    println!(
                        "{RED}Unknown command: {}{RESET}. Type ':help' for available commands.",
                        trimmed
                    );
                    continue;
                }
            }
        }

        if trimmed.starts_with('/') {
            if let Some(slash_cmd) = agent::orchestrator::SlashCommand::parse(trimmed) {
                match slash_cmd {
                    agent::orchestrator::SlashCommand::Help => {
                        println!("{BOLD}{CYAN}Available Slash Commands:{RESET}");
                        println!(
                            "  {CYAN}/goal <prompt>{RESET}     Run long-running autonomous task"
                        );
                        println!("  {CYAN}/plan <prompt>{RESET}     Formulate structured plan before editing");
                        println!("  {CYAN}/fix <prompt>{RESET}      Targeted error diagnostics and minimal repair");
                        println!("  {CYAN}/grill-me <topic>{RESET}  Interactive interview to resolve ambiguity");
                        println!("  {CYAN}/compact{RESET}           Compact prior tool results in memory");
                        println!("  {CYAN}/tasks{RESET}             List background processes and daemons");
                        println!("  {CYAN}/schedule <spec>{RESET}   Schedule timer or recurring cron check");
                        println!(
                            "  {CYAN}/learn <rule>{RESET}      Persist learned rule to workspace"
                        );
                        continue;
                    }
                    agent::orchestrator::SlashCommand::Compact => {
                        let (total_msgs, before_len, after_len) = agent.compact_memory();
                        let saved = before_len.saturating_sub(after_len);
                        println!(
                            "{GREEN}✓ Memory compacted:{RESET} {total_msgs} messages. Total size: {before_len} chars -> {after_len} chars ({saved} chars saved)."
                        );
                        continue;
                    }
                    agent::orchestrator::SlashCommand::Tasks => {
                        let tasks = agent.task_manager().list();
                        if tasks.is_empty() {
                            println!("{DIM}No active background tasks or daemons.{RESET}");
                        } else {
                            println!("{BOLD}{CYAN}Background Tasks ({}):{RESET}", tasks.len());
                            for t in tasks {
                                let status_color = if t.status.starts_with("running") {
                                    YELLOW
                                } else if t.status.contains("code: 0") {
                                    GREEN
                                } else {
                                    RED
                                };
                                println!(
                                    "  • [{status_color}{}{RESET}] {BOLD}{}{RESET} | daemon: {} | elapsed: {}s | cmd: {DIM}{}{RESET}",
                                    t.status, t.task_id, t.is_daemon, t.elapsed_seconds, t.command
                                );
                            }
                        }
                        continue;
                    }
                    cmd => {
                        if let Some(intent) = cmd.intent_override() {
                            agent.set_intent(intent);
                        }
                        let enriched = cmd.enriched_prompt();
                        println!("{BOLD}{CYAN}⚡ Executing Slash Directive:{RESET} {DIM}{trimmed}{RESET}");
                        let res = execute_turn(
                            &mut agent,
                            &enriched,
                            config.verbose,
                            config.auto_approve,
                        );
                        match res {
                            Ok(_) => {
                                session_json = agent.export_session_json().ok();
                            }
                            Err(e) => {
                                eprintln!("{RED}Agent error: {}{RESET}", e);
                            }
                        }
                        println!();
                        continue;
                    }
                }
            } else {
                eprintln!("{RED}Unknown slash command: {}{RESET}. Type '/help' for available slash commands.", trimmed);
                continue;
            }
        }

        let res = execute_turn(&mut agent, trimmed, config.verbose, config.auto_approve);
        match res {
            Ok(_) => {
                session_json = agent.export_session_json().ok();
            }
            Err(e) => {
                eprintln!("{RED}Agent error: {}{RESET}", e);
            }
        }
        println!();
    }
}

pub fn main() {
    let config = match parse_args() {
        Ok(c) => c,
        Err(code) => std::process::exit(code),
    };

    if let Some(ref prompt) = config.prompt {
        let agent_opt = create_configured_agent(&config);

        let mut agent = match agent_opt {
            Some(a) => a,
            None => {
                print_missing_keys_error();
                std::process::exit(1);
            }
        };

        if config.verbose {
            println!("{BOLD}{CYAN}ZENE Agent One-Shot Runner{RESET}");
            println!(
                "Provider: {GREEN}{}{RESET} | Model: {GREEN}{}{RESET} | Workspace: {BLUE}{}{RESET}",
                config.provider,
                config.model,
                config.workspace.display()
            );
            println!("Prompt: {BOLD}\"{}\"{RESET}\n", prompt);
        }

        let auto_app = config.auto_approve || true;
        let _ = execute_turn(&mut agent, &prompt, config.verbose, auto_app);
    } else {
        run_interactive(config);
    }
}
