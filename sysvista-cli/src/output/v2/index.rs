use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{RelationshipId, ScopeId, Snapshot};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ScopeIndex {
    pub scopes: Vec<ScopeSlice>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ScopeSlice {
    pub scope_id: ScopeId,
    pub child_ids: Vec<String>,
    pub owner_map: BTreeMap<String, String>,
    pub crossing_relationship_ids: Vec<RelationshipId>,
}

impl ScopeIndex {
    pub fn from_snapshot(snapshot: &Snapshot) -> Self {
        let entity_scopes: BTreeMap<_, _> = snapshot
            .entities
            .iter()
            .map(|entity| (entity.id.clone(), entity.scope_id.clone()))
            .collect();
        let mut by_scope: BTreeMap<ScopeId, ScopeSlice> = BTreeMap::new();
        for file in &snapshot.source_files {
            let scope_id = super::scope_id(&file.id);
            by_scope
                .entry(scope_id.clone())
                .or_insert_with(|| ScopeSlice {
                    scope_id,
                    child_ids: Vec::new(),
                    owner_map: BTreeMap::new(),
                    crossing_relationship_ids: Vec::new(),
                })
                .child_ids
                .push(file.id.0.clone());
        }
        for entity in &snapshot.entities {
            let scope = by_scope
                .entry(entity.scope_id.clone())
                .or_insert_with(|| ScopeSlice {
                    scope_id: entity.scope_id.clone(),
                    child_ids: Vec::new(),
                    owner_map: BTreeMap::new(),
                    crossing_relationship_ids: Vec::new(),
                });
            scope.child_ids.push(entity.id.0.clone());
            if let Some(owner) = &entity.owner_id {
                scope.owner_map.insert(entity.id.0.clone(), owner.0.clone());
            }
        }
        for relationship in &snapshot.relationships {
            let (id, source, target, _, _) = relationship.sort_key();
            let source_scope = entity_scopes.get(source);
            let target_scope = entity_scopes.get(target);
            if source_scope != target_scope {
                if let Some(scope) = source_scope.and_then(|scope| by_scope.get_mut(scope)) {
                    scope.crossing_relationship_ids.push(id.clone());
                }
                if let Some(scope) = target_scope.and_then(|scope| by_scope.get_mut(scope)) {
                    scope.crossing_relationship_ids.push(id.clone());
                }
            }
        }
        for scope in by_scope.values_mut() {
            scope.child_ids.sort();
            scope.child_ids.dedup();
            scope.crossing_relationship_ids.sort();
            scope.crossing_relationship_ids.dedup();
        }
        Self {
            scopes: by_scope.into_values().collect(),
        }
    }
}
