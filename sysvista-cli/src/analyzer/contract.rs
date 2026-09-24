use serde::{Deserialize, Serialize};

pub const CONTRACT_VERSION: u32 = 2;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalyzeRequest {
    pub contract_version: u32,
    pub root: String,
    pub files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tsconfig: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalyzerEntity {
    pub name: String,
    pub ownership_chain: String,
    pub declaration_kind: String,
    pub file: String,
    pub discriminator: usize,
    #[serde(default)]
    pub is_local: bool,
    /// Key of the nearest enclosing declaration; absent for top-level declarations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_key: Option<String>,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
    #[serde(default)]
    pub attributes: serde_json::Map<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalyzerRelationship {
    pub kind: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    pub origin: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<AnalyzerSpan>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalyzerSpan {
    pub file: String,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalyzerUnresolved {
    pub source: String,
    pub name: String,
    pub span: AnalyzerSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalyzerDiagnostic {
    pub message: String,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<AnalyzerSpan>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalyzerPayload {
    pub name: String,
    #[serde(default)]
    pub producers: Vec<String>,
    #[serde(default)]
    pub consumers: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalyzeResponse {
    pub contract_version: u32,
    pub analyzer_version: String,
    #[serde(default)]
    pub entities: Vec<AnalyzerEntity>,
    #[serde(default)]
    pub relationships: Vec<AnalyzerRelationship>,
    #[serde(default)]
    pub unresolved: Vec<AnalyzerUnresolved>,
    #[serde(default)]
    pub diagnostics: Vec<AnalyzerDiagnostic>,
    #[serde(default)]
    pub payloads: Vec<AnalyzerPayload>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Handshake {
    pub contract_version: u32,
    pub analyzer_version: String,
}
