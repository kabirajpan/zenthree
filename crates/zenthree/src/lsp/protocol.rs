use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 Request Envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest<T> {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: T,
}

impl<T: Serialize> JsonRpcRequest<T> {
    pub fn new(id: u64, method: impl Into<String>, params: T) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC 2.0 Notification Envelope (no id)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification<T> {
    pub jsonrpc: String,
    pub method: String,
    pub params: T,
}

impl<T: Serialize> JsonRpcNotification<T> {
    pub fn new(method: impl Into<String>, params: T) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
        }
    }
}

/// Encode a payload with HTTP-like Content-Length header
pub fn encode_lsp_message<T: Serialize>(val: &T) -> Result<Vec<u8>, serde_json::Error> {
    let body = serde_json::to_string(val)?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    let mut bytes = header.into_bytes();
    bytes.extend_from_slice(body.as_bytes());
    Ok(bytes)
}

/// Parse Content-Length from an LSP header buffer
pub fn parse_content_length(header: &str) -> Option<usize> {
    for line in header.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Content-Length:") {
            if let Ok(len) = rest.trim().parse::<usize>() {
                return Some(len);
            }
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────
// Standard LSP Types
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentIdentifier {
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedTextDocumentIdentifier {
    pub uri: String,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentItem {
    pub uri: String,
    #[serde(rename = "languageId")]
    pub language_id: String,
    pub version: u64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentContentChangeEvent {
    pub text: String,
}

// ─────────────────────────────────────────────────────────────
// Diagnostics
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

impl<'de> Deserialize<'de> for DiagnosticSeverity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = u32::deserialize(deserializer)?;
        match val {
            1 => Ok(DiagnosticSeverity::Error),
            2 => Ok(DiagnosticSeverity::Warning),
            3 => Ok(DiagnosticSeverity::Information),
            4 => Ok(DiagnosticSeverity::Hint),
            _ => Ok(DiagnosticSeverity::Information),
        }
    }
}

impl Serialize for DiagnosticSeverity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(*self as u32)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Option<DiagnosticSeverity>,
    pub code: Option<serde_json::Value>,
    pub source: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishDiagnosticsParams {
    pub uri: String,
    pub version: Option<u64>,
    pub diagnostics: Vec<Diagnostic>,
}

// ─────────────────────────────────────────────────────────────
// Completions
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionItemKind {
    Text = 1,
    Method = 2,
    Function = 3,
    Constructor = 4,
    Field = 5,
    Variable = 6,
    Class = 7,
    Interface = 8,
    Module = 9,
    Property = 10,
    Unit = 11,
    Value = 12,
    Enum = 13,
    Keyword = 14,
    Snippet = 15,
    Color = 16,
    File = 17,
    Reference = 18,
    Folder = 19,
    EnumMember = 20,
    Constant = 21,
    Struct = 22,
    Event = 23,
    Operator = 24,
    TypeParameter = 25,
}

impl<'de> Deserialize<'de> for CompletionItemKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = u32::deserialize(deserializer)?;
        match val {
            1 => Ok(CompletionItemKind::Text),
            2 => Ok(CompletionItemKind::Method),
            3 => Ok(CompletionItemKind::Function),
            4 => Ok(CompletionItemKind::Constructor),
            5 => Ok(CompletionItemKind::Field),
            6 => Ok(CompletionItemKind::Variable),
            7 => Ok(CompletionItemKind::Class),
            8 => Ok(CompletionItemKind::Interface),
            9 => Ok(CompletionItemKind::Module),
            10 => Ok(CompletionItemKind::Property),
            11 => Ok(CompletionItemKind::Unit),
            12 => Ok(CompletionItemKind::Value),
            13 => Ok(CompletionItemKind::Enum),
            14 => Ok(CompletionItemKind::Keyword),
            15 => Ok(CompletionItemKind::Snippet),
            16 => Ok(CompletionItemKind::Color),
            17 => Ok(CompletionItemKind::File),
            18 => Ok(CompletionItemKind::Reference),
            19 => Ok(CompletionItemKind::Folder),
            20 => Ok(CompletionItemKind::EnumMember),
            21 => Ok(CompletionItemKind::Constant),
            22 => Ok(CompletionItemKind::Struct),
            23 => Ok(CompletionItemKind::Event),
            24 => Ok(CompletionItemKind::Operator),
            25 => Ok(CompletionItemKind::TypeParameter),
            _ => Ok(CompletionItemKind::Text),
        }
    }
}

impl Serialize for CompletionItemKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(*self as u32)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: Option<CompletionItemKind>,
    pub detail: Option<String>,
    pub documentation: Option<serde_json::Value>,
    #[serde(rename = "insertText")]
    pub insert_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CompletionResponse {
    List(CompletionList),
    Array(Vec<CompletionItem>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionList {
    #[serde(rename = "isIncomplete")]
    pub is_incomplete: bool,
    pub items: Vec<CompletionItem>,
}

// ─────────────────────────────────────────────────────────────
// Lifecycle Parameters
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "processId")]
    pub process_id: Option<u32>,
    #[serde(rename = "clientInfo")]
    pub client_info: Option<ClientInfo>,
    #[serde(rename = "rootUri")]
    pub root_uri: Option<String>,
    pub capabilities: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidOpenTextDocumentParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidChangeTextDocumentParams {
    #[serde(rename = "textDocument")]
    pub text_document: VersionedTextDocumentIdentifier,
    #[serde(rename = "contentChanges")]
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidCloseTextDocumentParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GotoDefinitionResponse {
    Scalar(Location),
    Array(Vec<Location>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GotoDefinitionParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkupContent {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HoverContents {
    Scalar(String),
    Markup(MarkupContent),
    Array(Vec<serde_json::Value>),
}

impl HoverContents {
    pub fn to_markdown_or_plain(&self) -> String {
        match self {
            HoverContents::Scalar(s) => s.clone(),
            HoverContents::Markup(m) => m.value.clone(),
            HoverContents::Array(arr) => {
                arr.iter()
                    .filter_map(|v| {
                        if let Some(s) = v.as_str() {
                            Some(s.to_string())
                        } else if let Ok(m) = serde_json::from_value::<MarkupContent>(v.clone()) {
                            Some(m.value)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n")
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hover {
    pub contents: HoverContents,
    pub range: Option<Range>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_and_parse_lsp_message() {
        let req = JsonRpcRequest::new(1, "initialize", serde_json::json!({"test": true}));
        let encoded = encode_lsp_message(&req).expect("Failed to encode");
        let encoded_str = String::from_utf8(encoded).expect("Not UTF-8");

        assert!(encoded_str.starts_with("Content-Length: "));
        assert!(encoded_str.contains("\r\n\r\n"));

        let len = parse_content_length(&encoded_str);
        assert!(len.is_some());
        assert!(len.unwrap() > 0);
    }

    #[test]
    fn test_diagnostic_severity_deserialization() {
        let json_diag = serde_json::json!({
            "range": {
                "start": { "line": 10, "character": 4 },
                "end": { "line": 10, "character": 12 }
            },
            "severity": 1,
            "message": "expected `;`"
        });

        let diag: Diagnostic = serde_json::from_value(json_diag).expect("Failed to parse diagnostic");
        assert_eq!(diag.range.start.line, 10);
        assert_eq!(diag.severity, Some(DiagnosticSeverity::Error));
        assert_eq!(diag.message, "expected `;`");
    }

    #[test]
    fn test_completion_response_deserialization() {
        let json_comp = serde_json::json!({
            "isIncomplete": false,
            "items": [
                {
                    "label": "clone",
                    "kind": 2,
                    "detail": "fn clone(&self) -> Self",
                    "insertText": "clone()"
                }
            ]
        });

        let resp: CompletionResponse = serde_json::from_value(json_comp).expect("Failed to parse completion");
        match resp {
            CompletionResponse::List(list) => {
                assert_eq!(list.items.len(), 1);
                assert_eq!(list.items[0].label, "clone");
                assert_eq!(list.items[0].kind, Some(CompletionItemKind::Method));
                assert_eq!(list.items[0].insert_text.as_deref(), Some("clone()"));
            }
            _ => panic!("Expected CompletionList"),
        }
    }

    #[test]
    fn test_goto_definition_response_deserialization() {
        let json_loc = serde_json::json!({
            "uri": "file:///path/to/file.rs",
            "range": {
                "start": { "line": 42, "character": 7 },
                "end": { "line": 42, "character": 15 }
            }
        });
        let resp: GotoDefinitionResponse = serde_json::from_value(json_loc).expect("Failed to parse scalar location");
        match resp {
            GotoDefinitionResponse::Scalar(loc) => {
                assert_eq!(loc.uri, "file:///path/to/file.rs");
                assert_eq!(loc.range.start.line, 42);
                assert_eq!(loc.range.start.character, 7);
            }
            _ => panic!("Expected scalar location"),
        }

        let json_arr = serde_json::json!([
            {
                "uri": "file:///path/to/def.rs",
                "range": {
                    "start": { "line": 1, "character": 0 },
                    "end": { "line": 1, "character": 5 }
                }
            }
        ]);
        let resp_arr: GotoDefinitionResponse = serde_json::from_value(json_arr).expect("Failed to parse array location");
        match resp_arr {
            GotoDefinitionResponse::Array(locs) => {
                assert_eq!(locs.len(), 1);
                assert_eq!(locs[0].uri, "file:///path/to/def.rs");
            }
            _ => panic!("Expected array location"),
        }
    }

    #[test]
    fn test_hover_deserialization() {
        let json_hover = serde_json::json!({
            "contents": {
                "kind": "markdown",
                "value": "```rust\npub fn hello()\n```\nSays hello"
            },
            "range": {
                "start": { "line": 5, "character": 4 },
                "end": { "line": 5, "character": 9 }
            }
        });

        let hover: Hover = serde_json::from_value(json_hover).expect("Failed to parse hover");
        assert_eq!(hover.contents.to_markdown_or_plain(), "```rust\npub fn hello()\n```\nSays hello");
        assert!(hover.range.is_some());
    }
}
