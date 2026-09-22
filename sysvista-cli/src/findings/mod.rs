pub mod cycles;
pub mod forbidden;
pub mod gaps;
pub mod membership;
pub mod unresolved;

use crate::output::v2::{Finding, Snapshot};

/// Derive canonical findings using only data contained in the snapshot.
pub fn derive(snapshot: &Snapshot) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(cycles::derive(snapshot));
    findings.extend(forbidden::derive(snapshot));
    findings.extend(membership::derive(snapshot));
    findings.extend(unresolved::derive(snapshot));
    findings.extend(gaps::derive(snapshot));
    findings.sort_by_key(|item| serde_json::to_string(item).unwrap_or_default());
    findings
}

pub(crate) fn navigation(snapshot: &Snapshot, entity_ids: &[crate::output::v2::EntityId]) -> crate::output::v2::NavigationTarget {
    let mut scopes = entity_ids.iter().filter_map(|id| snapshot.entities.iter().find(|e| &e.id == id).map(|e| e.scope_id.clone()));
    let first = scopes.next();
    let common = first.filter(|scope| scopes.all(|candidate| candidate == *scope));
    let scope_id = common.or_else(|| snapshot.projections.iter().find(|p| p.kind == "repository").map(|p| p.scope_id.clone()))
        .or_else(|| snapshot.projections.first().map(|p| p.scope_id.clone())).unwrap_or_else(|| crate::output::v2::ScopeId("scope:unknown".into()));
    crate::output::v2::NavigationTarget { scope_id, entity_id: (entity_ids.len() == 1).then(|| entity_ids[0].clone()) }
}

pub(crate) fn rule(snapshot: &Snapshot, rule_id: &str, message: String, mut entities: Vec<crate::output::v2::EntityId>, mut files: Vec<crate::output::v2::FileId>, mut relationships: Vec<crate::output::v2::RelationshipId>, mut sites: Vec<crate::output::v2::SourceSpan>) -> Finding {
    entities.sort(); entities.dedup(); files.sort(); files.dedup(); relationships.sort(); relationships.dedup();
    sites.sort_by(|a,b| (&a.file_id,a.start_line,a.start_column,a.end_line,a.end_column).cmp(&(&b.file_id,b.start_line,b.start_column,b.end_line,b.end_column)));
    let affected: Vec<&str> = entities.iter().map(|id| id.as_ref()).chain(files.iter().map(|id| id.as_ref())).chain(relationships.iter().map(|id| id.as_ref())).collect();
    let id = crate::output::v2::stable_id("finding", &[rule_id, &affected.join("|")]);
    let navigation_target = navigation(snapshot, &entities);
    Finding::Rule { id, rule_id: rule_id.into(), message, affected_entity_ids: entities, affected_file_ids: files, relationship_ids: relationships, supporting_sites: sites, navigation_target }
}
