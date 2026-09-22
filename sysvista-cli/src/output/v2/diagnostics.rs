use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{EntityId, RelationshipId, SourceSpan};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Diagnostic {
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
