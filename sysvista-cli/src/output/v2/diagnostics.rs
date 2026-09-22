use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{EntityId, FileId, RelationshipId, SourceSpan};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Diagnostic {
    AnalyzerUnavailable {
        message: String,
        severity: String,
    },
    AnalyzerContractMismatch {
        message: String,
        severity: String,
    },
    AnalyzerIssue {
        message: String,
        severity: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<SourceSpan>,
    },
    UnreadableFile {
        id: String,
        path: String,
        message: String,
    },
    FailedFile {
        id: String,
        path: String,
        message: String,
    },
    Warning {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<SourceSpan>,
    },
    MembershipConflict {
        id: String,
        file_id: FileId,
        path: String,
        modules: Vec<String>,
        message: String,
    },
    DanglingReference {
        id: String,
        relationship_id: RelationshipId,
        missing_entity_id: EntityId,
        message: String,
    },
    Coverage {
        id: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<FileId>,
    },
    Contradiction {
        id: String,
        relationship_ids: Vec<RelationshipId>,
        message: String,
    },
    StaleEvidence {
        id: String,
        evidence_id: String,
        message: String,
    },
    SourceUnavailable {
        id: String,
        file_id: FileId,
        path: String,
        message: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Finding {
    Entity {
        entity_id: EntityId,
        message: String,
    },
    Relationship {
        relationship_id: RelationshipId,
        message: String,
    },
    Project {
        message: String,
    },
}
