use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{EntityId, RelationshipId, SourceSpan};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Relationship {
    Imports {
        id: RelationshipId,
        source: EntityId,
        target: EntityId,
        origin: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        evidence_id: Option<String>,
    },
    References {
        id: RelationshipId,
        source: EntityId,
        target: EntityId,
        origin: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        evidence_id: Option<String>,
    },
    Calls {
        id: RelationshipId,
        source: EntityId,
        target: EntityId,
        origin: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        evidence_id: Option<String>,
    },
    Contains {
        id: RelationshipId,
        source: EntityId,
        target: EntityId,
        origin: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        evidence_id: Option<String>,
    },
    DependsOn {
        id: RelationshipId,
        source: EntityId,
        target: EntityId,
        origin: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        evidence_id: Option<String>,
    },
    Handles {
        id: RelationshipId,
        source: EntityId,
        target: EntityId,
        origin: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        evidence_id: Option<String>,
    },
    Persists {
        id: RelationshipId,
        source: EntityId,
        target: EntityId,
        origin: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        evidence_id: Option<String>,
    },
    Transforms {
        id: RelationshipId,
        source: EntityId,
        target: EntityId,
        origin: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        evidence_id: Option<String>,
    },
    Consumes {
        id: RelationshipId,
        source: EntityId,
        target: EntityId,
        origin: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        evidence_id: Option<String>,
    },
    Produces {
        id: RelationshipId,
        source: EntityId,
        target: EntityId,
        origin: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        evidence_id: Option<String>,
    },
    Dispatches {
        id: RelationshipId,
        source: EntityId,
        target: EntityId,
        origin: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        evidence_id: Option<String>,
    },
    InvokesPrompt {
        id: RelationshipId,
        source: EntityId,
        target: EntityId,
        origin: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        evidence_id: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct UnresolvedReference {
    pub name: String,
    pub source: EntityId,
    pub span: SourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Evidence {
    Source {
        id: String,
        span: SourceSpan,
    },
    Text {
        id: String,
        value: String,
    },
    Analyzer {
        id: String,
        analyzer: String,
        detail: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Claim {
    pub id: String,
    pub subject: EntityId,
    pub predicate: String,
    pub object: serde_json::Value,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PayloadContract {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
    #[serde(default)]
    pub producer_ids: Vec<EntityId>,
    #[serde(default)]
    pub consumer_ids: Vec<EntityId>,
}
