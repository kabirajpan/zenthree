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
}

impl Default for CliConfig {
    fn default() -> Self {
        let has_antigravity_token = agent::external_provider::AntigravityAuthManager::with_default_config()
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
        }
    }
}

fn print_version() {
    println!("zene {}", env!("CARGO_PKG_VERSION"));
}

fn print_missing_keys_error() {
    eprintln!("\n{RED}{BOLD}✗ Error: No AI provider credentials found.{RESET}");
    eprintln!("\nZENE can operate via API keys or your Google Account:\n");
    eprintln!("  1. {CYAN}{BOLD}Google Antigravity{RESET} (Use your own Google ID via OAuth):");
    eprintln!("     Login directly: {GREEN}zene --login{RESET} or run with {GREEN}zene -p antigravity{RESET}\n");
    eprintln!("  2. {CYAN}{BOLD}Google Gemini{RESET} (API Key):");
    eprintln!("     Get key: {DIM}https://aistudio.google.com/app/apikey{RESET}");
    eprintln!("     Set it:  {GREEN}export GEMINI_API_KEY=\"your-key-here\"{RESET}\n");
    eprintln!("  3. {CYAN}{BOLD}Groq{RESET} (Ultra-low latency inference):");
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
    println!("  {GREEN}-p, --provider <NAME>{RESET}     AI provider: 'antigravity', 'gemini', or 'groq'");
    println!("  {GREEN}-m, --model <NAME>{RESET}        Model identifier (e.g. gemini-3.8-flash-high, gemini-3.5-flash-lite)");
    println!("  {GREEN}-w, --workspace <DIR>{RESET}     Target workspace directory (default: current directory)");
    println!("  {GREEN}--login{RESET}                   Log in with your Google Account / Google ID for Antigravity");
    println!("  {GREEN}--list-models{RESET}             List recommended models for each provider");
    println!("  {GREEN}-V, --version{RESET}             Print version information");
    println!("  {GREEN}-h, --help{RESET}                Display this help message\n");
    println!("{BOLD}INTERACTIVE COMMANDS:{RESET} (when launched without [PROMPT])");
    println!("  {YELLOW}:help{RESET}                     Show command list");
    println!("  {YELLOW}:login{RESET}                    Log in with your Google Account");
    println!("  {YELLOW}:verbose{RESET}                  Toggle verbose diagnostic mode on/off");
    println!("  {YELLOW}:model <NAME>{RESET}             Switch model on the fly");
    println!("  {YELLOW}:clear{RESET}                    Reset conversation memory");
    println!("  {YELLOW}:quit / :exit{RESET}             Exit the interactive session\n");
}

fn print_models() {
    println!("{BOLD}{CYAN}Available & Tested Models:{RESET}\n");
    println!("{BOLD}Google Antigravity Models (Authenticate with your Google ID):{RESET}");
    for m in agent::external_provider::available_models() {
        println!("  • {GREEN}{:<24}{RESET} {}", m.id, m.description);
    }
    println!("\n{BOLD}Gemini API Models (Via GEMINI_API_KEY):{RESET}");
    println!("  • {GREEN}gemini-3.5-flash-lite{RESET}   Ultra-lightweight, high throughput (Default)");
    println!("  • {GREEN}gemini-3.5-flash{RESET}        Fast, full function-calling, generous daily quota\n");
    println!("{BOLD}Groq Models (Via GROQ_API_KEY):{RESET}");
    println!("  • {GREEN}openai/gpt-oss-120b{RESET}     High reasoning, full tool calling");
    println!("  • {GREEN}openai/gpt-oss-20b{RESET}      Fast lightweight reasoning\n");
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
            "--login" | "login" => {
                println!("{BOLD}{CYAN}Initiating Google Antigravity OAuth Login...{RESET}");
                let auth = agent::external_provider::AntigravityAuthManager::with_default_config();
                match auth.login_interactive() {
                    Ok(()) => {
                        println!("\n{BOLD}{GREEN}✓ Successfully authenticated via Antigravity!{RESET}");
                        println!("  Session saved to: {DIM}{}{RESET}", agent::external_provider::antigravity_cli::auth::default_token_path().display());
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
            "-p" | "--provider" => {
                if i + 1 < args.len() {
                    config.provider = args[i + 1].to_lowercase();
                    if config.provider == "antigravity" && config.model == "gemini-3.5-flash-lite" {
                        config.model = "gemini-3.8-flash-high".to_string();
                    }
                    i += 1;
                }
            }
            "-m" | "--model" => {
                if i + 1 < args.len() {
                    config.model = args[i + 1].clone();
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
                    format!("{}...\n[truncated {} bytes]", &res_str[..600], res_str.len() - 600)
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
                println!("  {GREEN}✓ {name}{RESET} {DIM}finished ({} lines output){RESET}", lines);
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

    let mut ask_approval = |_req: ApprovalRequest| ApprovalDecision::Approve;

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

    let sys_msg = format!(
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
    );
    agent.push_system_message(sys_msg);
    Some(agent)
}

fn run_interactive(mut config: CliConfig) {
    println!("{BOLD}{CYAN}═══════════════════════════════════════════════════════════════{RESET}");
    println!("{BOLD}{CYAN}           ZENE Agent Interactive Developer CLI                {RESET}");
    println!("{BOLD}{CYAN}═══════════════════════════════════════════════════════════════{RESET}");
    println!("  {BOLD}Provider:{RESET}   {GREEN}{}{RESET}", config.provider);
    println!("  {BOLD}Model:{RESET}      {GREEN}{}{RESET}", config.model);
    println!("  {BOLD}Workspace:{RESET}  {BLUE}{}{RESET}", config.workspace.display());
    println!("  {BOLD}Verbose:{RESET}    {}", if config.verbose { format!("{YELLOW}ON{RESET}") } else { format!("{DIM}OFF{RESET}") });
    println!("{DIM}Type your instructions or ':help' for commands. ':quit' to exit.{RESET}\n");

    let agent_opt = create_configured_agent(&config);

    let mut agent = match agent_opt {
        Some(a) => a,
        None => {
            print_missing_keys_error();
            return;
        }
    };


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
                ":login" => {
                    println!("{BOLD}{CYAN}Initiating Google Antigravity OAuth Login...{RESET}");
                    let auth = agent::external_provider::AntigravityAuthManager::with_default_config();
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
                    println!("Verbose mode: {}", if config.verbose { format!("{YELLOW}ON{RESET}") } else { format!("{DIM}OFF{RESET}") });
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
                        if let Some(mut fresh) = agent::create_agent_for_model_and_workspace(
                            &config.provider,
                            &config.model,
                            &config.workspace,
                        ) {
                            if let Some(ref json) = session_json {
                                let _ = fresh.import_session_json(json);
                            }
                            agent = fresh;
                            println!("{GREEN}Switched model to: {}{RESET}", config.model);
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
                        println!("{GREEN}Switched workspace to: {}{RESET}", config.workspace.display());
                    }
                    continue;
                }
                _ => {
                    println!("{RED}Unknown command: {}{RESET}. Type ':help' for available commands.", trimmed);
                    continue;
                }
            }
        }

        let res = execute_turn(&mut agent, trimmed, config.verbose);
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
            println!("Provider: {GREEN}{}{RESET} | Model: {GREEN}{}{RESET} | Workspace: {BLUE}{}{RESET}",
                config.provider, config.model, config.workspace.display());
            println!("Prompt: {BOLD}\"{}\"{RESET}\n", prompt);
        }

        let _ = execute_turn(&mut agent, &prompt, config.verbose);
    } else {
        run_interactive(config);
    }
}
