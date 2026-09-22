use crate::output::v2::{Diagnostic, Finding, Snapshot};

pub fn derive(snapshot: &Snapshot) -> Vec<Finding> {
    let mut out=Vec::new();
    for diagnostic in &snapshot.diagnostics {
        if let Diagnostic::MembershipConflict { file_id, path, .. } = diagnostic {
            let entities=snapshot.entities.iter().filter(|e| &e.file_id==file_id).map(|e|e.id.clone()).collect();
            out.push(super::rule(snapshot,"membership_conflict",format!("{path} has conflicting module membership"),entities,vec![file_id.clone()],Vec::new(),Vec::new()));
        }
    }
    if let Some(module)=snapshot.modules.iter().find(|m|m.name=="Unassigned") {
        for file in &module.file_ids {
            if snapshot.diagnostics.iter().any(|d|matches!(d,Diagnostic::MembershipConflict{file_id,..} if file_id==file)){continue;}
            let entities=snapshot.entities.iter().filter(|e|&e.file_id==file).map(|e|e.id.clone()).collect();
            out.push(super::rule(snapshot,"unassigned_file","file is not assigned to a logical module".into(),entities,vec![file.clone()],Vec::new(),Vec::new()));
        }
    }
    out
}
