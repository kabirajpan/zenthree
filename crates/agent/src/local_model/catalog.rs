use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CatalogModel {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub parameter_size: &'static str,
    pub quantization: &'static str,
    pub size_bytes: u64,
    pub size_display: &'static str,
    pub download_url: &'static str,
    pub filename: &'static str,
    pub ollama_tag: &'static str,
    pub context_length: u32,
    pub recommended_for: &'static str,
}

pub const MODEL_CATALOG: &[CatalogModel] = &[
    CatalogModel {
        id: "qwen2.5-coder-1.5b-q4",
        name: "Qwen 2.5 Coder 1.5B (Q4_K_M)",
        description: "Ultra-fast, lightweight code generation, inline edits, and code explanation.",
        parameter_size: "1.5B",
        quantization: "Q4_K_M",
        size_bytes: 986_000_000,
        size_display: "986 MB",
        download_url: "https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf",
        filename: "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf",
        ollama_tag: "qwen2.5-coder:1.5b",
        context_length: 32768,
        recommended_for: "Low-spec machines, ultra-fast responses",
    },
    CatalogModel {
        id: "qwen2.5-coder-7b-q4",
        name: "Qwen 2.5 Coder 7B (Q4_K_M)",
        description: "State-of-the-art coding benchmark leader for 7B models. Multi-turn reasoning & tool calling.",
        parameter_size: "7B",
        quantization: "Q4_K_M",
        size_bytes: 4_680_000_000,
        size_display: "4.68 GB",
        download_url: "https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct-GGUF/resolve/main/qwen2.5-coder-7b-instruct-q4_k_m.gguf",
        filename: "qwen2.5-coder-7b-instruct-q4_k_m.gguf",
        ollama_tag: "qwen2.5-coder:7b",
        context_length: 32768,
        recommended_for: "Best overall local coding assistant (8GB+ RAM)",
    },
    CatalogModel {
        id: "deepseek-r1-distill-qwen-1.5b-q4",
        name: "DeepSeek R1 Distill Qwen 1.5B (Q4_K_M)",
        description: "Reasoning and Chain-of-Thought distilled model in a compact 1.5B footprint.",
        parameter_size: "1.5B",
        quantization: "Q4_K_M",
        size_bytes: 1_120_000_000,
        size_display: "1.12 GB",
        download_url: "https://huggingface.co/bartowski/DeepSeek-R1-Distill-Qwen-1.5B-GGUF/resolve/main/DeepSeek-R1-Distill-Qwen-1.5B-Q4_K_M.gguf",
        filename: "deepseek-r1-distill-qwen-1.5b-q4_k_m.gguf",
        ollama_tag: "deepseek-r1:1.5b",
        context_length: 32768,
        recommended_for: "Step-by-step logic and planning on low resources",
    },
    CatalogModel {
        id: "deepseek-r1-distill-qwen-7b-q4",
        name: "DeepSeek R1 Distill Qwen 7B (Q4_K_M)",
        description: "Exceptional mathematical and code reasoning capabilities distilled from DeepSeek R1.",
        parameter_size: "7B",
        quantization: "Q4_K_M",
        size_bytes: 4_680_000_000,
        size_display: "4.68 GB",
        download_url: "https://huggingface.co/bartowski/DeepSeek-R1-Distill-Qwen-7B-GGUF/resolve/main/DeepSeek-R1-Distill-Qwen-7B-Q4_K_M.gguf",
        filename: "deepseek-r1-distill-qwen-7b-q4_k_m.gguf",
        ollama_tag: "deepseek-r1:7b",
        context_length: 32768,
        recommended_for: "Advanced architecture & complex debugging",
    },
    CatalogModel {
        id: "llama-3.2-3b-q4",
        name: "Llama 3.2 3B Instruct (Q4_K_M)",
        description: "Meta's highly capable edge model. Balanced speed and instruction following.",
        parameter_size: "3B",
        quantization: "Q4_K_M",
        size_bytes: 2_020_000_000,
        size_display: "2.02 GB",
        download_url: "https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF/resolve/main/Llama-3.2-3B-Instruct-Q4_K_M.gguf",
        filename: "llama-3.2-3b-instruct-q4_k_m.gguf",
        ollama_tag: "llama3.2:3b",
        context_length: 32768,
        recommended_for: "General conversation and balanced coding",
    },
    CatalogModel {
        id: "smollm2-1.7b-q4",
        name: "SmolLM2 1.7B Instruct (Q4_K_M)",
        description: "Hugging Face's ultra-compact, high efficiency small language model.",
        parameter_size: "1.7B",
        quantization: "Q4_K_M",
        size_bytes: 1_060_000_000,
        size_display: "1.06 GB",
        download_url: "https://huggingface.co/HuggingFaceTB/SmolLM2-1.7B-Instruct-GGUF/resolve/main/smollm2-1.7b-instruct-q4_k_m.gguf",
        filename: "smollm2-1.7b-instruct-q4_k_m.gguf",
        ollama_tag: "smollm2:1.7b",
        context_length: 8192,
        recommended_for: "Fastest responses on laptops and lightweight devices",
    },
];

pub fn find_catalog_model(id_or_tag_or_file: &str) -> Option<&'static CatalogModel> {
    let query = id_or_tag_or_file.to_lowercase();
    MODEL_CATALOG.iter().find(|m| {
        m.id.to_lowercase() == query
            || m.ollama_tag.to_lowercase() == query
            || m.filename.to_lowercase() == query
            || m.name.to_lowercase().contains(&query)
    })
}
