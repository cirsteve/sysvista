use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{EntityId, RelationshipId, ScopeId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Projection {
    pub id: String,
    pub name: String,
    pub scope_id: ScopeId,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_scope_id: Option<ScopeId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default)]
    pub entity_ids: Vec<EntityId>,
    #[serde(default)]
    pub relationship_ids: Vec<RelationshipId>,
}
