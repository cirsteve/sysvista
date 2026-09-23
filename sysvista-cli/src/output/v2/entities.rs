use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{EntityId, FileId, ModuleId, ScopeId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SourceSpan {
    pub file_id: FileId,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AnalysisStatus {
    /// Supported but not (yet) analyzed.
    None,
    /// No analyzer supports the file's language; not an analysis gap.
    Unsupported,
    Parsed { analyzer: String },
    Failed { message: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SourceFile {
    pub id: FileId,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub analysis: AnalysisStatus,
    #[serde(default, skip_serializing)]
    pub content_hash: Option<String>,
    #[serde(default, skip_serializing)]
    pub byte_length: Option<u64>,
    #[serde(default, skip_serializing)]
    pub line_count: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CodeEntity {
    pub id: EntityId,
    pub name: String,
    pub qualified_name: String,
    pub declaration_kind: String,
    #[serde(default)]
    pub is_local: bool,
    pub file_id: FileId,
    pub scope_id: ScopeId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<EntityId>,
    pub span: SourceSpan,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LogicalModule {
    pub id: ModuleId,
    pub name: String,
    pub scope_id: ScopeId,
    #[serde(default)]
    pub selectors: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub file_ids: Vec<FileId>,
    #[serde(default)]
    pub entity_ids: Vec<EntityId>,
}
