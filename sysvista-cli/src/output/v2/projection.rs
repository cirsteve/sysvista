use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{EntityId, RelationshipId, ScopeId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Projection {
    pub id: String,
    pub name: String,
    pub scope_id: ScopeId,
    #[serde(default)]
    pub entity_ids: Vec<EntityId>,
    #[serde(default)]
    pub relationship_ids: Vec<RelationshipId>,
}
