pub mod types;
pub mod agents;
pub mod orchestrator;
pub mod tools;
pub mod providers;
pub mod context;
pub mod acp;
pub mod goose;
pub mod environment;
pub mod permissions;
pub mod skills;
pub mod rules;
pub mod context_management;
pub mod diff;
pub mod protocols;

// Root convenience re-exports
pub use protocols::{
    McpClient, McpTransport, McpStdioTransport, McpSseTransport,
    McpTool, McpResource, McpPrompt, CallToolResult, InitializeResult,
    AcpInboundCommand,
};
pub use diff::{
    MatrixDiff, StreamingDiff, DiffOp, DiffHunk, BufferEditSession,
    EditSessionStatus, TurnCheckpoint, CheckpointTimeline, RevertReport,
};
pub use context_management::{
    ContextManager, TokenAccountant, LargeOutputSpiller, SpilledOutputResult,
    ToolPairPruner, PruningReport, ConversationCompactor, CompactionReport,
    DEFAULT_LARGE_OUTPUT_THRESHOLD, DEFAULT_COMPACTION_THRESHOLD_RATIO,
};
pub use environment::{AgentEnvironment, WorkspaceJail, ProtectedPathGuard, SecretScrubber, SecurityViolation};
pub use skills::{SkillsRegistry, Skill, SkillMetadata, SkillScope, SkillCatalog, SkillSummary};
pub use rules::{RulesRegistry, RuleDocument, RuleScope, RuleScanner, render_rules_prompt};
pub use permissions::{
    HardcodedSecurityRules, ShellCommandParser, check_hardcoded_security_rules,
    HARDCODED_SECURITY_DENIAL, PermissionEngine, ToolPermissionDecision,
    UserPermissionResponse, ToolPermissionMode, PermissionPolicyTable,
};
pub use types::{
    AiMessage, ChatMessage, EditedFileSummary, ExploredItem, ProjectEnvironment,
    ProjectLanguage, UserIntent, WorkspaceDetectionContext, CancellationToken,
    classify_user_intent, AgentEvent, PendingPermissionRequest,
};

pub use agents::SubAgentRole;

pub use orchestrator::{
    AgentMission, AgentTask, TaskStatus, create_mission_for_prompt,
    AgentLoop, AgentLoopContext, AgentBackend,
    AgentStateMachine, AgentTurnState, FsmEvent, AgentSession, TurnRecord, TurnStatus,
    SystemPromptBuilder, SteeringController, SteeringSignal,
};

pub use acp::{AcpClient, AcpTransport, AcpAdapter};
pub use goose::{GooseSupervisor, GooseBinaryStatus, GooseConfig};

pub use tools::{
    AgentToolCall, AgentToolResult, DetectorAgent, FileEditAction, SearchReplaceBlock,
    TargetFileModification, apply_search_replace, execute_agent_tool, execute_agent_tool_with_bridge,
    execute_agent_tool_with_provider, McpToolBridge, RegisteredMcpTool,
    all_tool_schemas_with_mcp,
    execute_detect_environment,
    execute_list_directory, execute_read_file, execute_run_command, execute_search_code,
    execute_git_status, execute_git_diff,
    extract_all_file_modifications, extract_json_tool_modifications,
    extract_tool_call_read_paths, parse_agent_tool_calls, parse_search_replace_blocks,
    parse_single_tool_call_value, parse_tool_call_values, ground_workspace_path,
};

pub use providers::{
    ModelProvider, ProviderRegistry, ModelRequest, ModelResponse, ModelToolCall, StreamChunk,
    TokenUsage, AnthropicProvider, OpenAiProvider, GeminiProvider, GroqProvider, LocalProvider,
    ProviderChain, ProviderError, call_gemini_api, call_gemini_api_with_model,
    call_groq_api, call_groq_api_messages, call_groq_api_messages_with_model,
    call_groq_api_with_model, classify_provider_error, load_gemini_api_key, load_groq_api_key,
};

pub use context::{
    build_autonomous_agent_prompt, build_discussion_guide_prompt,
    build_self_healing_prompt, filter_compiler_errors,
    run_project_compiler_check, run_project_compiler_check_with_env, run_project_tests,
    truncate_output,
};
