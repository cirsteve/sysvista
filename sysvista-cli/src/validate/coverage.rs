use std::collections::BTreeMap;
use crate::output::v2::{self, Diagnostic, Snapshot};

pub fn validate(snapshot: &Snapshot) -> Vec<Diagnostic> {
    let mut counts: BTreeMap<_, usize> = snapshot.source_files.iter().map(|f| (f.id.clone(), 0)).collect();
    for module in &snapshot.modules { for file in &module.file_ids { *counts.entry(file.clone()).or_default() += 1; } }
    counts.into_iter().filter(|(_, count)| *count != 1).map(|(file_id, count)| Diagnostic::Coverage {
        id: v2::stable_id("diagnostic", &["coverage", file_id.as_ref(), &count.to_string()]), file_id: Some(file_id.clone()),
        message: format!("file {} belongs to {count} logical modules; expected exactly one", file_id.as_ref()),
    }).collect()
}
