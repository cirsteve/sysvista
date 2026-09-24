use crate::output::v2::{AnalysisStatus, Diagnostic, Finding, Snapshot};
use super::Index;

pub fn derive(snapshot: &Snapshot, index: &Index) -> Vec<Finding> {
    let mut out=Vec::new();
    // Unsupported files were never meant to be analyzed; only supported files that were
    // not, or failed, are gaps.
    for file in &snapshot.source_files {
        if matches!(file.analysis,AnalysisStatus::None|AnalysisStatus::Failed{..}) {
            out.push(super::rule(index,"analysis_gap",format!("{} was not successfully analyzed",file.path),index.file_entities(&file.id),vec![file.id.clone()],Vec::new(),Vec::new()));
        }
    }
    if snapshot.diagnostics.iter().any(|d|matches!(d,Diagnostic::AnalyzerUnavailable{..}|Diagnostic::AnalyzerContractMismatch{..})) {
        out.push(super::rule(index,"analyzer_unavailable","an analyzer was unavailable".into(),Vec::new(),Vec::new(),Vec::new(),Vec::new()));
    }
    out
}
