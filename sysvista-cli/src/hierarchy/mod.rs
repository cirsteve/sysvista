pub mod logical;
pub mod ownership;
pub mod physical;

use crate::{
    discovery::{Config, Inventory},
    output::v2::{Diagnostic, Snapshot},
};

pub fn derive(snapshot: &mut Snapshot, inventory: &Inventory, config: &Config) {
    let owners: std::collections::BTreeMap<_, _> = snapshot.entities.iter()
        .map(|e| (e.id.clone(), e.name.clone())).collect();
    for entity in &mut snapshot.entities {
        if let Some(owner) = &entity.owner_id {
            if owners.get(owner).is_some_and(|name| name != "<module>") {
                entity.scope_id = crate::output::v2::ScopeId(crate::output::v2::stable_id(
                    "scope", &["symbol", owner.as_ref()]));
            }
        }
    }
    snapshot.forbidden_dependencies = config.forbidden_dependencies.iter().map(|rule| crate::output::v2::ForbiddenDependencyRule { from: rule.from.clone(), to: rule.to.clone() }).collect();
    snapshot.projections = physical::derive(
        &snapshot.manifest.repository,
        inventory,
        &snapshot.source_files,
        &snapshot.entities,
    );
    snapshot.modules = logical::derive(&snapshot.manifest.repository, config);
    snapshot.diagnostics.extend(ownership::assign(
        &mut snapshot.modules,
        &snapshot.source_files,
        &snapshot.entities,
    ));
    for module in &snapshot.modules {
        snapshot.projections.push(crate::output::v2::Projection {
            id: crate::output::v2::stable_id("projection", &["logical", module.id.as_ref()]),
            name: module.name.clone(),
            scope_id: module.scope_id.clone(),
            kind: "logical_module".into(),
            parent_scope_id: Some(snapshot.manifest.root_scope_id.clone()),
            tags: module.tags.clone(),
            entity_ids: module.entity_ids.clone(),
            relationship_ids: Vec::new(),
        });
    }
    snapshot.projections.sort_by(|a, b| a.id.cmp(&b.id));
}

pub fn membership_conflicts(snapshot: &Snapshot) -> impl Iterator<Item = &Diagnostic> {
    snapshot
        .diagnostics
        .iter()
        .filter(|d| matches!(d, Diagnostic::MembershipConflict { .. }))
}
