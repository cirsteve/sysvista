use crate::output::v2::{AnalysisStatus, Diagnostic, Finding, Snapshot};

pub fn derive(snapshot: &Snapshot) -> Vec<Finding> {
    let mut out=Vec::new();
    for file in &snapshot.source_files {
        if matches!(file.analysis,AnalysisStatus::None|AnalysisStatus::Failed{..}) {
            let entities=snapshot.entities.iter().filter(|e|e.file_id==file.id).map(|e|e.id.clone()).collect();
            out.push(super::rule(snapshot,"analysis_gap",format!("{} was not successfully analyzed",file.path),entities,vec![file.id.clone()],Vec::new(),Vec::new()));
        }
    }
    if snapshot.diagnostics.iter().any(|d|matches!(d,Diagnostic::AnalyzerUnavailable{..}|Diagnostic::AnalyzerContractMismatch{..})) {
        out.push(super::rule(snapshot,"analyzer_unavailable","an analyzer was unavailable".into(),Vec::new(),Vec::new(),Vec::new(),Vec::new()));
    }
    out
}
