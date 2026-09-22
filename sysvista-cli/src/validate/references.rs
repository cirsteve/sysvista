use std::collections::BTreeSet;
use crate::output::v2::{self, Diagnostic, Snapshot};

pub fn validate(snapshot: &Snapshot) -> Vec<Diagnostic> {
    let entities: BTreeSet<_> = snapshot.entities.iter().map(|entity| &entity.id).collect();
    let mut out = Vec::new();
    for relationship in &snapshot.relationships {
        let (id, source, target, _, _) = super::relationship_parts(relationship);
        for missing in [source, target].into_iter().filter(|entity| !entities.contains(entity)) {
            out.push(Diagnostic::DanglingReference { id: v2::stable_id("diagnostic", &["dangling_reference", id.as_ref(), missing.as_ref()]),
                relationship_id: id.clone(), missing_entity_id: missing.clone(), message: format!("relationship {} references missing entity {}", id.as_ref(), missing.as_ref()) });
        }
    }
    out
}
