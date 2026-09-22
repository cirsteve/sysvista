use std::collections::{BTreeMap, BTreeSet};

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
    #[serde(default)]
    pub child_scope_ids: Vec<ScopeId>,
    pub child_ids: Vec<String>,
    pub owner_map: BTreeMap<String, String>,
    pub crossing_relationship_ids: Vec<RelationshipId>,
}

impl ScopeIndex {
    pub fn from_snapshot(snapshot: &Snapshot) -> Self {
        let entity_owners: BTreeMap<_, _> = snapshot
            .entities
            .iter()
            .filter_map(|entity| {
                entity
                    .owner_id
                    .as_ref()
                    .map(|owner| (entity.id.clone(), owner.clone()))
            })
            .collect();
        let mut member_scopes: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
        let mut by_scope: BTreeMap<ScopeId, ScopeSlice> = BTreeMap::new();
        for projection in &snapshot.projections {
            by_scope
                .entry(projection.scope_id.clone())
                .or_insert_with(|| ScopeSlice {
                    scope_id: projection.scope_id.clone(),
                    child_scope_ids: Vec::new(),
                    child_ids: Vec::new(),
                    owner_map: BTreeMap::new(),
                    crossing_relationship_ids: Vec::new(),
                });
            if let Some(parent) = &projection.parent_scope_id {
                by_scope
                    .entry(parent.clone())
                    .or_insert_with(|| ScopeSlice {
                        scope_id: parent.clone(),
                        child_scope_ids: Vec::new(),
                        child_ids: Vec::new(),
                        owner_map: BTreeMap::new(),
                        crossing_relationship_ids: Vec::new(),
                    })
                    .child_scope_ids
                    .push(projection.scope_id.clone());
            }
            let scope = by_scope
                .get_mut(&projection.scope_id)
                .expect("projection scope was inserted");
            for id in &projection.entity_ids {
                scope.child_ids.push(id.0.clone());
                if let Some(owner) = entity_owners.get(id) {
                    scope.owner_map.insert(id.0.clone(), owner.0.clone());
                }
                member_scopes
                    .entry(id.clone())
                    .or_default()
                    .insert(projection.scope_id.clone());
            }
        }
        for file in &snapshot.source_files {
            let scope_id = super::scope_id(&file.id);
            by_scope
                .entry(scope_id.clone())
                .or_insert_with(|| ScopeSlice {
                    scope_id,
                    child_scope_ids: Vec::new(),
                    child_ids: Vec::new(),
                    owner_map: BTreeMap::new(),
                    crossing_relationship_ids: Vec::new(),
                })
                .child_ids
                .push(file.id.0.clone());
        }
        for entity in &snapshot.entities {
            member_scopes
                .entry(entity.id.clone())
                .or_default()
                .insert(entity.scope_id.clone());
            let scope = by_scope
                .entry(entity.scope_id.clone())
                .or_insert_with(|| ScopeSlice {
                    scope_id: entity.scope_id.clone(),
                    child_scope_ids: Vec::new(),
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
            let scopes = member_scopes
                .get(source)
                .into_iter()
                .chain(member_scopes.get(target))
                .flatten()
                .cloned()
                .collect::<BTreeSet<_>>();
            for scope_id in scopes {
                if let Some(scope) = by_scope.get_mut(&scope_id) {
                    scope.crossing_relationship_ids.push(id.clone());
                }
            }
        }
        for scope in by_scope.values_mut() {
            scope.child_scope_ids.sort();
            scope.child_scope_ids.dedup();
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
