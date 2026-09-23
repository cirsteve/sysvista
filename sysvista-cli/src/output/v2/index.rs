use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{EntityId, FileId, ModuleId, RelationshipId, ScopeId, Snapshot};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ScopeIndex {
    pub scopes: Vec<ScopeSlice>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ScopeSlice {
    pub scope_id: ScopeId,
    #[serde(default)]
    pub child_scope_ids: Vec<ScopeId>,
    pub children: Vec<ScopeChild>,
    pub owner_map: BTreeMap<String, String>,
    pub crossing_relationship_ids: Vec<RelationshipId>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ScopeChild {
    Directory { scope_id: ScopeId },
    File { file_id: FileId, scope_id: ScopeId },
    Module { module_id: ModuleId, scope_id: ScopeId },
    Symbol { entity_id: EntityId, scope_id: Option<ScopeId> },
}

impl ScopeChild {
    pub fn scope_id(&self) -> Option<&ScopeId> {
        match self {
            Self::Directory { scope_id } | Self::File { scope_id, .. } | Self::Module { scope_id, .. } => Some(scope_id),
            Self::Symbol { scope_id, .. } => scope_id.as_ref(),
        }
    }
}


impl ScopeIndex {
    pub fn from_snapshot(snapshot: &Snapshot) -> Self {
        if let Some(index) = &snapshot.scope_index { return index.clone(); }
        Self::with_extensions(snapshot, &["ts", "tsx", "js", "jsx", "mjs", "cjs", "rs", "py"].map(str::to_owned))
    }

    pub fn with_extensions(snapshot: &Snapshot, extensions: &[String]) -> Self {
        let mut slices: BTreeMap<ScopeId, ScopeSlice> = snapshot.projections.iter()
            .map(|p| (p.scope_id.clone(), ScopeSlice {
                scope_id: p.scope_id.clone(), child_scope_ids: vec![], children: vec![],
                owner_map: BTreeMap::new(), crossing_relationship_ids: vec![],
            })).collect();
        let files: BTreeMap<_, _> = snapshot.source_files.iter().map(|f| (f.id.clone(), f)).collect();
        let files_by_scope: BTreeMap<_, _> = snapshot.source_files.iter()
            .map(|file| (super::scope_id(&file.id), file)).collect();
        let entities: BTreeMap<_, _> = snapshot.entities.iter().map(|e| (e.id.clone(), e)).collect();
        let projections_by_scope: BTreeMap<_, _> = snapshot.projections.iter()
            .map(|projection| (&projection.scope_id, projection)).collect();
        let non_module_ids: BTreeSet<_> = snapshot.entities.iter()
            .filter(|entity| entity.name != "<module>").map(|entity| entity.id.clone()).collect();
        let nested_owners: BTreeSet<_> = snapshot.entities.iter().filter_map(|e| e.owner_id.clone())
            .filter(|id| non_module_ids.contains(id))
            .collect();
        let mut relationships_by_entity: BTreeMap<EntityId, BTreeSet<RelationshipId>> = BTreeMap::new();
        for relationship in &snapshot.relationships {
            let (id, source, target, _, _) = relationship.sort_key();
            relationships_by_entity.entry(source.clone()).or_default().insert(id.clone());
            relationships_by_entity.entry(target.clone()).or_default().insert(id.clone());
        }
        let visible = |path: &str| {
            let ext = std::path::Path::new(path).extension().and_then(|x| x.to_str()).unwrap_or("");
            extensions.iter().any(|x| x == ext)
        };
        for p in &snapshot.projections {
            let Some(parent_id) = &p.parent_scope_id else { continue };
            let Some(parent) = slices.get_mut(parent_id) else { continue };
            let child = match p.kind.as_str() {
                "file" => files_by_scope.get(&p.scope_id).copied()
                    .filter(|f| visible(&f.path))
                    .map(|f| ScopeChild::File { file_id: f.id.clone(), scope_id: p.scope_id.clone() }),
                "symbol" | "logical_module" => None,
                _ => Some(ScopeChild::Directory { scope_id: p.scope_id.clone() }),
            };
            if let Some(child) = child {
                parent.children.push(child);
                parent.child_scope_ids.push(p.scope_id.clone());
            }
        }
        if let Some(root) = slices.get_mut(&snapshot.manifest.root_scope_id) {
            for module in &snapshot.modules {
                root.children.push(ScopeChild::Module { module_id: module.id.clone(), scope_id: module.scope_id.clone() });
                root.child_scope_ids.push(module.scope_id.clone());
            }
        }
        for module in &snapshot.modules {
            if let Some(slice) = slices.get_mut(&module.scope_id) {
                for id in &module.file_ids {
                    if let Some(file) = files.get(id).filter(|f| visible(&f.path)) {
                        let scope_id = super::scope_id(&file.id);
                        slice.children.push(ScopeChild::File { file_id: file.id.clone(), scope_id: scope_id.clone() });
                        slice.child_scope_ids.push(scope_id);
                    }
                }
            }
        }
        for entity in &snapshot.entities {
            if entity.name == "<module>" { continue; }
            if let Some(slice) = slices.get_mut(&entity.scope_id) {
                let child_scope = nested_owners.contains(&entity.id)
                    .then(|| ScopeId(super::stable_id("scope", &["symbol", entity.id.as_ref()])));
                slice.children.push(ScopeChild::Symbol { entity_id: entity.id.clone(), scope_id: child_scope.clone() });
                if let Some(id) = child_scope { slice.child_scope_ids.push(id); }
            }
        }
        for p in &snapshot.projections {
            let Some(slice) = slices.get_mut(&p.scope_id) else { continue };
            let visible_ids: BTreeSet<String> = slice.children.iter().map(|child| match child {
                ScopeChild::Directory { scope_id } => scope_id.0.clone(),
                ScopeChild::File { file_id, .. } => file_id.0.clone(),
                ScopeChild::Module { module_id, .. } => module_id.0.clone(),
                ScopeChild::Symbol { entity_id, .. } => entity_id.0.clone(),
            }).collect();
            let directory_by_path: BTreeMap<_, _> = slice.children.iter().filter_map(|child| match child {
                ScopeChild::Directory { scope_id } => projections_by_scope.get(scope_id)
                    .map(|projection| (projection.name.as_str(), scope_id.0.as_str())),
                _ => None,
            }).collect();
            let file_representatives: BTreeMap<_, _> = p.entity_ids.iter()
                .filter_map(|id| entities.get(id).map(|entity| &entity.file_id))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .filter_map(|file_id| {
                    let file = files.get(file_id)?;
                    let mut parent = std::path::Path::new(&file.path).parent();
                    while let Some(path) = parent {
                        let name = path.to_str().unwrap_or("");
                        if let Some(scope_id) = directory_by_path.get(name) { return Some((file_id.clone(), (*scope_id).to_owned())); }
                        parent = path.parent();
                    }
                    None
                }).collect();
            for id in &p.entity_ids {
                let mut cursor = id.clone();
                let mut visited = BTreeSet::new();
                while visited.insert(cursor.clone()) {
                    if visible_ids.contains(cursor.as_ref()) {
                        slice.owner_map.insert(id.0.clone(), cursor.0);
                        break;
                    }
                    let Some(entity) = entities.get(&cursor) else { break };
                    if let Some(owner) = &entity.owner_id {
                        cursor = owner.clone();
                    } else if visible_ids.contains(entity.file_id.as_ref()) {
                        slice.owner_map.insert(id.0.clone(), entity.file_id.0.clone());
                        break;
                    } else {
                        if let Some(scope_id) = file_representatives.get(&entity.file_id) {
                            slice.owner_map.insert(id.0.clone(), scope_id.clone());
                        }
                        break;
                    }
                }
            }
            for entity_id in &p.entity_ids {
                if let Some(ids) = relationships_by_entity.get(entity_id) {
                    slice.crossing_relationship_ids.extend(ids.iter().cloned());
                }
            }
        }
        for slice in slices.values_mut() {
            slice.children.sort(); slice.children.dedup();
            slice.child_scope_ids.sort(); slice.child_scope_ids.dedup();
            slice.crossing_relationship_ids.sort(); slice.crossing_relationship_ids.dedup();
        }
        Self { scopes: slices.into_values().collect() }
    }
}
