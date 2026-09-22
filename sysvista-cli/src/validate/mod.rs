pub mod contradictions;
pub mod coverage;
pub mod references;
pub mod staleness;

use crate::output::v2::{Diagnostic, Snapshot, ValidationSummary};

/// Validate a complete snapshot without filesystem or process access.
pub fn validate(snapshot: &Snapshot) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(references::validate(snapshot));
    diagnostics.extend(coverage::validate(snapshot));
    diagnostics.extend(contradictions::validate(snapshot));
    diagnostics.extend(staleness::validate(snapshot));
    diagnostics.sort_by_key(|item| serde_json::to_string(item).unwrap_or_default());
    diagnostics
}

pub fn summary(diagnostics: &[Diagnostic]) -> ValidationSummary {
    let errors = diagnostics.iter().filter(|d| matches!(d, Diagnostic::DanglingReference { .. } | Diagnostic::Contradiction { .. } | Diagnostic::StaleEvidence { .. })).count() as u64;
    ValidationSummary { diagnostics: diagnostics.len() as u64, errors, warnings: diagnostics.len() as u64 - errors }
}

pub(crate) fn relationship_parts(relationship: &crate::output::v2::Relationship) -> (&crate::output::v2::RelationshipId, &crate::output::v2::EntityId, &crate::output::v2::EntityId, &'static str, &str) {
    relationship.sort_key()
}
