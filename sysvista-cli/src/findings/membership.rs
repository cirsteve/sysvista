use std::collections::BTreeSet;
use crate::output::v2::{Diagnostic, Finding, Snapshot};
use super::Index;

pub fn derive(snapshot: &Snapshot, index: &Index) -> Vec<Finding> {
    let mut out=Vec::new();
    let mut conflicted=BTreeSet::new();
    for diagnostic in &snapshot.diagnostics {
        if let Diagnostic::MembershipConflict { file_id, path, .. } = diagnostic {
            conflicted.insert(file_id);
            out.push(super::rule(index,"membership_conflict",format!("{path} has conflicting module membership"),index.file_entities(file_id),vec![file_id.clone()],Vec::new(),Vec::new()));
        }
    }
    // Without configured modules every file is unassigned; that is not a finding.
    let configured = snapshot.modules.iter().any(|m| m.name != "Unassigned");
    if let Some(module)=snapshot.modules.iter().find(|m|m.name=="Unassigned").filter(|_| configured) {
        for file in module.file_ids.iter().filter(|file| !conflicted.contains(file)) {
            out.push(super::rule(index,"unassigned_file","file is not assigned to a logical module".into(),index.file_entities(file),vec![file.clone()],Vec::new(),Vec::new()));
        }
    }
    out
}
