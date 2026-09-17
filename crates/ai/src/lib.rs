pub mod types;
pub mod agents;
pub mod orchestrator;
pub mod tools;
pub mod providers;
pub mod context;
pub mod deliberation;
pub mod session;

// Root convenience re-exports
pub use types::{
    AiMessage, ChatMessage, EditedFileSummary, ExploredItem, ProjectEnvironment,
    ProjectLanguage, UserIntent, WorkspaceDetectionContext, CancellationToken,
    classify_user_intent, AgentEvent,
};

pub use agents::{
    SubAgentRole, SubAgent, ZeneAgent, PlannerAgent, ExplorerAgent, BuilderAgent, TesterAgent,
};

pub use orchestrator::{
    AgentMission, AgentTask, TaskStatus, create_mission_for_prompt,
    AgentLoop, AgentLoopContext,
};

pub use tools::{
    AgentToolCall, AgentToolResult, DetectorAgent, FileEditAction, SearchReplaceBlock,
    TargetFileModification, apply_search_replace, execute_agent_tool, execute_agent_tool_with_provider,
    execute_detect_environment,
    execute_list_directory, execute_read_file, execute_run_command, execute_search_code,
    execute_git_status, execute_git_diff,
    extract_all_file_modifications, extract_json_tool_modifications,
    extract_tool_call_read_paths, parse_agent_tool_calls, parse_search_replace_blocks,
    parse_single_tool_call_value,
};

pub use providers::{
    ProviderChain, ProviderError, call_gemini_api, call_gemini_api_with_model,
    call_groq_api, call_groq_api_messages, call_groq_api_messages_with_model,
    call_groq_api_with_model, classify_provider_error, load_gemini_api_key, load_groq_api_key,
};

pub use context::{
    build_autonomous_agent_prompt, build_discussion_guide_prompt, build_project_context,
    build_self_healing_prompt, extract_prompt_keywords, filter_compiler_errors,
    run_project_compiler_check, run_project_compiler_check_with_env, run_project_tests,
    truncate_output, FileMatch, ProjectContextSummary, SymbolLocation,
};

pub use deliberation::DeliberationBridge;

pub use session::{
    AgentSession, SessionArtifact, SessionSummary, delete_session, format_bytes,
    format_relative_time, get_session_dir, list_saved_sessions,
};
