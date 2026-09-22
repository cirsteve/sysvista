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

impl Relationship {
    pub fn sort_key(&self) -> (&RelationshipId, &EntityId, &EntityId, &'static str, &str) {
        macro_rules! fields {
            ($kind:literal, $id:ident, $source:ident, $target:ident, $origin:ident) => {
                ($id, $source, $target, $kind, $origin.as_str())
            };
        }
        match self {
            Self::Imports {
                id,
                source,
                target,
                origin,
                ..
            } => fields!("imports", id, source, target, origin),
            Self::References {
                id,
                source,
                target,
                origin,
                ..
            } => fields!("references", id, source, target, origin),
            Self::Calls {
                id,
                source,
                target,
                origin,
                ..
            } => fields!("calls", id, source, target, origin),
            Self::Contains {
                id,
                source,
                target,
                origin,
                ..
            } => fields!("contains", id, source, target, origin),
            Self::DependsOn {
                id,
                source,
                target,
                origin,
                ..
            } => fields!("depends_on", id, source, target, origin),
            Self::Handles {
                id,
                source,
                target,
                origin,
                ..
            } => fields!("handles", id, source, target, origin),
            Self::Persists {
                id,
                source,
                target,
                origin,
                ..
            } => fields!("persists", id, source, target, origin),
            Self::Transforms {
                id,
                source,
                target,
                origin,
                ..
            } => fields!("transforms", id, source, target, origin),
            Self::Consumes {
                id,
                source,
                target,
                origin,
                ..
            } => fields!("consumes", id, source, target, origin),
            Self::Produces {
                id,
                source,
                target,
                origin,
                ..
            } => fields!("produces", id, source, target, origin),
            Self::Dispatches {
                id,
                source,
                target,
                origin,
                ..
            } => fields!("dispatches", id, source, target, origin),
            Self::InvokesPrompt {
                id,
                source,
                target,
                origin,
                ..
            } => fields!("invokes_prompt", id, source, target, origin),
        }
    }
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
