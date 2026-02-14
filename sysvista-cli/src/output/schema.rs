use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ComponentKind {
    Model,
    Service,
    Transport,
    Transform,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransportProtocol {
    Http,
    Grpc,
    Websocket,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceLocation {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_start: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_end: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DetectedComponent {
    pub id: String,
    pub name: String,
    pub kind: ComponentKind,
    pub language: String,
    pub source: SourceLocation,
    pub metadata: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport_protocol: Option<TransportProtocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_fields: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub produces: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DetectedEdge {
    pub from_id: String,
    pub to_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanStats {
    pub files_scanned: u64,
    pub files_skipped: u64,
    pub scan_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SysVistaOutput {
    pub version: String,
    pub scanned_at: String,
    pub root_dir: String,
    pub project_name: String,
    pub detected_languages: Vec<String>,
    pub components: Vec<DetectedComponent>,
    pub edges: Vec<DetectedEdge>,
    pub scan_stats: ScanStats,
}
