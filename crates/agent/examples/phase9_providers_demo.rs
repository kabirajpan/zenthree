//! Phase 9 Multi-Model Streaming Providers Demo
//!
//! Demonstrates the unified streaming model drivers and resilience engine:
//! 1. Unified ModelRequest & Multi-Model Payload Construction (Anthropic, OpenAI, Gemini, Groq, Local)
//! 2. Claude 3.7 Sonnet Extended Thinking & Token Streaming
//! 3. ProviderRegistry with Fallback Priority Chaining & Automatic Exponential Backoff
//! 4. Full Native + MCP Tool Schema Integration into Model Requests
//!
//! Run with:
//! cargo run --example phase9_providers_demo -p agent

use agent::providers::{
    AnthropicProvider, GeminiProvider, GroqProvider, LocalProvider, ModelProvider, ModelRequest,
    ModelResponse, OpenAiProvider, ProviderRegistry, ProviderRole, StreamChunk, TokenUsage,
};
use agent::tools::all_tool_schemas;

struct MockStreamingProvider {
    _name: String,
    simulated_thinking: String,
    simulated_content: String,
}

impl ModelProvider for MockStreamingProvider {
    fn stream(
        &self,
        _req: &ModelRequest,
        on_chunk: &mut dyn FnMut(StreamChunk),
    ) -> Result<ModelResponse, String> {
        // Stream thinking tokens
        for word in self.simulated_thinking.split_whitespace() {
            on_chunk(StreamChunk::Thinking {
                delta: format!("{} ", word),
            });
        }

        // Stream output tokens
        for word in self.simulated_content.split_whitespace() {
            on_chunk(StreamChunk::Text {
                delta: format!("{} ", word),
            });
        }

        on_chunk(StreamChunk::Usage {
            prompt_tokens: 1540,
            completion_tokens: 380,
        });

        on_chunk(StreamChunk::Done);

        Ok(ModelResponse {
            content: self.simulated_content.clone(),
            thinking: Some(self.simulated_thinking.clone()),
            tool_calls: Vec::new(),
            usage: TokenUsage::new(1540, 380),
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("⚡ ZENTHREE AGENT PROVIDERS DEMO (PHASE 9: MULTI-MODEL DRIVERS)");
    println!("============================================================\n");

    // -----------------------------------------------------------------------
    // Section 1: Unified ModelRequest with Native + MCP Tool Schemas
    // -----------------------------------------------------------------------
    println!("--- 1. Unified ModelRequest Construction ---");

    let tools = all_tool_schemas();
    let unified_req = ModelRequest::new("claude-3-7-sonnet-20250219")
        .with_system("You are Zenthree autonomous agent with surgical precision.")
        .with_message(ProviderRole::User, "Implement token validation middleware")
        .with_tools(tools.clone())
        .with_thinking(4096);

    println!("Unified Request Summary:");
    println!("  Model: {}", unified_req.model);
    println!("  System Prompt Length: {} chars", unified_req.system.as_ref().map(|s| s.len()).unwrap_or(0));
    println!("  Messages Count: {}", unified_req.messages.len());
    println!("  Tools Attached: {} schemas (Native + MCP)", unified_req.tools.len());
    println!("  Thinking Budget: {:?} tokens\n", unified_req.thinking_budget);

    // -----------------------------------------------------------------------
    // Section 2: Model Payload Generation across 5 Providers
    // -----------------------------------------------------------------------
    println!("--- 2. Multi-Model Payload Adapters ---");

    // A. Anthropic (Claude 3.7 Sonnet)
    let ant_provider = AnthropicProvider::new("sk-ant-sample-key");
    let ant_payload = ant_provider.build_payload(&unified_req);
    println!("✓ Anthropic Claude Payload:");
    println!("  Model: {}", ant_payload["model"]);
    println!("  Thinking Config: {}", ant_payload["thinking"]);
    println!("  Tools Configured: {} definitions", ant_payload["tools"].as_array().map(|a| a.len()).unwrap_or(0));

    // B. OpenAI (GPT-4o)
    let openai_provider = OpenAiProvider::new("sk-openai-sample-key");
    let mut openai_req = unified_req.clone();
    openai_req.model = "gpt-4o".to_string();
    let openai_payload = openai_provider.build_payload(&openai_req);
    println!("\n✓ OpenAI Payload:");
    println!("  Model: {}", openai_payload["model"]);
    println!("  Stream: {}", openai_payload["stream"]);
    println!("  Messages Count: {}", openai_payload["messages"].as_array().map(|a| a.len()).unwrap_or(0));

    // C. Google Gemini (Gemini 2.5 Flash)
    let gemini_provider = GeminiProvider::new("gemini-sample-key");
    let mut gemini_req = unified_req.clone();
    gemini_req.model = "gemini-2.5-flash".to_string();
    let gemini_payload = gemini_provider.build_payload(&gemini_req);
    println!("\n✓ Google Gemini Payload:");
    println!("  Contents Count: {}", gemini_payload["contents"].as_array().map(|a| a.len()).unwrap_or(0));
    println!("  Has Function Declarations: {}", gemini_payload["tools"].is_array());

    // D. Groq (Fast Llama 3.3 70B)
    let _groq_provider = GroqProvider::new("gsk-sample-key");
    println!("\n✓ Groq High-Speed Driver initialized for Llama 3.3 70B & DeepSeek R1");

    // E. Local Ollama / Llama.cpp Offline Runner
    let _local_provider = LocalProvider::new(None);
    println!("✓ Local Offline Inference Driver initialized (target: http://localhost:11434)");

    // -----------------------------------------------------------------------
    // Section 3: Live SSE Streaming Simulation (Thinking & Token Chunks)
    // -----------------------------------------------------------------------
    println!("\n--- 3. Real-Time Streaming Simulation (Thinking + Output Chunks) ---");

    let mock_driver = MockStreamingProvider {
        _name: "Mock-Claude-3.7".to_string(),
        simulated_thinking: "Analyzing token validation logic. We need to inspect Bearer header and verify HMAC signature.".to_string(),
        simulated_content: "I have implemented the JWT token validation middleware with signature checks.".to_string(),
    };

    print!("[THINKING CHUNKS]: ");
    let mut accumulated_thinking = String::new();
    let mut accumulated_tokens = String::new();

    let stream_result = mock_driver.stream(&unified_req, &mut |chunk| {
        match chunk {
            StreamChunk::Thinking { delta } => {
                accumulated_thinking.push_str(&delta);
                print!("{}", delta);
            }
            StreamChunk::Text { delta } => {
                accumulated_tokens.push_str(&delta);
            }
            StreamChunk::Usage { prompt_tokens, completion_tokens } => {
                println!("\n[USAGE TELEMETRY]: Prompt: {} tokens, Completion: {} tokens", prompt_tokens, completion_tokens);
            }
            StreamChunk::Done => {
                println!("[STREAM FINISHED]");
            }
            _ => {}
        }
    })?;

    println!("\n[ACCUMULATED CONTENT CHUNKS]:");
    println!("  {}\n", stream_result.content);

    // -----------------------------------------------------------------------
    // Section 4: ProviderRegistry Resilience & Fallback Chain
    // -----------------------------------------------------------------------
    println!("--- 4. ProviderRegistry Fallback & Resilience Chain ---");

    let mut registry = ProviderRegistry::new();
    registry.register("primary_anthropic", Box::new(MockStreamingProvider {
        _name: "Anthropic".to_string(),
        simulated_thinking: "Reviewing AST...".to_string(),
        simulated_content: "Resolved using Claude 3.7 Sonnet".to_string(),
    }));

    registry.set_fallback_order(vec![
        "primary_anthropic".to_string(),
        "fallback_gemini".to_string(),
        "offline_local".to_string(),
    ]);

    println!("✓ Registry configured with 3-tier fallback order: Anthropic -> Gemini -> Local");
    let fallback_res = registry.stream_with_fallback(&unified_req, &mut |_| {})?;
    println!("✓ Fallback Driver Executed Successfully: {}", fallback_res.content);

    println!("\n============================================================");
    println!("🎉 PHASE 9 PROVIDERS DEMO COMPLETED SUCCESSFULLY!");
    println!("============================================================");

    Ok(())
}
