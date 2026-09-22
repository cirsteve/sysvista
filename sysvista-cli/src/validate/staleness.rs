use std::collections::BTreeMap;
use crate::output::v2::{self, Diagnostic, Snapshot};

pub fn validate(snapshot: &Snapshot) -> Vec<Diagnostic> {
    let files: BTreeMap<_,_> = snapshot.source_files.iter().map(|f| (&f.id, f)).collect();
    let mut out = Vec::new();
    for evidence in &snapshot.evidence {
        let Some((id, span, evidence_hash)) = evidence.source() else { continue };
        let Some(file) = files.get(&span.file_id) else { continue };
        let outside = file.line_count.is_some_and(|lines| span.start_line == 0 || span.end_line > lines || span.start_line > span.end_line);
        let mismatch = evidence_hash.zip(file.content_hash.as_deref()).is_some_and(|(a,b)| a != b);
        if outside || mismatch {
            out.push(Diagnostic::StaleEvidence { id: v2::stable_id("diagnostic", &["stale_evidence", id]), evidence_id: id.into(),
                message: if outside { "evidence span is outside the source file".into() } else { "evidence content hash does not match the source file".into() } });
        }
    }
    out
}
