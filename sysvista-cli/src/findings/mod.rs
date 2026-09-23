pub(crate) mod cycles;
pub(crate) mod forbidden;
pub(crate) mod gaps;
pub(crate) mod membership;
pub(crate) mod unresolved;

use std::collections::HashMap;

use crate::output::v2::{CodeEntity, EntityId, FileId, Finding, NavigationTarget, RelationshipId, ScopeId, Snapshot, SourceSpan};

/// Lookups every rule needs, built once so no rule rescans all entities per file or finding.
pub(crate) struct Index<'a> {
    entities: HashMap<&'a EntityId, &'a CodeEntity>,
    entities_by_file: HashMap<&'a FileId, Vec<EntityId>>,
    fallback_scope: ScopeId,
}

impl<'a> Index<'a> {
    pub fn new(snapshot: &'a Snapshot) -> Self {
        let mut entities_by_file: HashMap<_, Vec<_>> = HashMap::new();
        for entity in &snapshot.entities {
            entities_by_file.entry(&entity.file_id).or_default().push(entity.id.clone());
        }
        let fallback_scope = snapshot.projections.iter().find(|p| p.kind == "repository").map(|p| p.scope_id.clone())
            .or_else(|| snapshot.projections.first().map(|p| p.scope_id.clone()))
            .unwrap_or_else(|| ScopeId("scope:unknown".into()));
        Self { entities: snapshot.entities.iter().map(|e| (&e.id, e)).collect(), entities_by_file, fallback_scope }
    }

    pub fn file_of(&self, entity: &EntityId) -> Option<&'a FileId> {
        self.entities.get(entity).map(|e| &e.file_id)
    }

    pub fn file_entities(&self, file: &FileId) -> Vec<EntityId> {
        self.entities_by_file.get(file).cloned().unwrap_or_default()
    }

    pub fn span(&self, entity: &EntityId) -> Option<SourceSpan> {
        self.entities.get(entity).map(|e| e.span.clone())
    }
}

/// Derive canonical findings using only data contained in the snapshot.
pub fn derive(snapshot: &Snapshot) -> Vec<Finding> {
    derive_with_exclusions(snapshot, &[])
}

pub fn derive_with_exclusions(snapshot: &Snapshot, excluded_reasons: &[String]) -> Vec<Finding> {
    let index = Index::new(snapshot);
    let mut findings = Vec::new();
    findings.extend(cycles::derive(snapshot, &index));
    findings.extend(forbidden::derive(snapshot, &index));
    findings.extend(membership::derive(snapshot, &index));
    findings.extend(unresolved::derive(snapshot, &index, excluded_reasons));
    findings.extend(gaps::derive(snapshot, &index));
    findings.sort_by_key(|item| serde_json::to_string(item).unwrap_or_default());
    findings
}

pub(crate) fn navigation(index: &Index, entity_ids: &[EntityId]) -> NavigationTarget {
    let mut scopes = entity_ids.iter().filter_map(|id| index.entities.get(id).map(|e| e.scope_id.clone()));
    let first = scopes.next();
    let common = first.filter(|scope| scopes.all(|candidate| candidate == *scope));
    NavigationTarget { scope_id: common.unwrap_or_else(|| index.fallback_scope.clone()), entity_id: (entity_ids.len() == 1).then(|| entity_ids[0].clone()) }
}

pub(crate) fn rule(index: &Index, rule_id: &str, message: String, mut entities: Vec<EntityId>, mut files: Vec<FileId>, mut relationships: Vec<RelationshipId>, mut sites: Vec<SourceSpan>) -> Finding {
    entities.sort(); entities.dedup(); files.sort(); files.dedup(); relationships.sort(); relationships.dedup();
    sites.sort_by(|a,b| (&a.file_id,a.start_line,a.start_column,a.end_line,a.end_column).cmp(&(&b.file_id,b.start_line,b.start_column,b.end_line,b.end_column)));
    sites.dedup();
    let affected: Vec<&str> = entities.iter().map(|id| id.as_ref()).chain(files.iter().map(|id| id.as_ref())).chain(relationships.iter().map(|id| id.as_ref())).collect();
    let id = crate::output::v2::stable_id("finding", &[rule_id, &affected.join("|")]);
    let navigation_target = navigation(index, &entities);
    Finding::Rule { id, rule_id: rule_id.into(), message, affected_entity_ids: entities, affected_file_ids: files, relationship_ids: relationships, supporting_sites: sites, navigation_target }
}
