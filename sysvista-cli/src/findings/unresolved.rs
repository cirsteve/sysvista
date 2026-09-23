use std::collections::BTreeMap;
use crate::output::v2::{Finding, Snapshot};
use super::Index;

/// Unresolved references grouped per file. Calls into default libraries and packages
/// never reach this list: the analyzer classifies them as external.
pub fn derive(snapshot: &Snapshot, index: &Index, excluded_reasons: &[String]) -> Vec<Finding> {
    let mut groups:BTreeMap<_,Vec<_>>=BTreeMap::new();
    for unresolved in &snapshot.unresolved_references {
        if unresolved.reason.as_deref().is_some_and(|reason| excluded_reasons.iter().any(|excluded| excluded == reason)) { continue; }
        if let Some(file)=index.file_of(&unresolved.source){groups.entry(file.clone()).or_default().push(unresolved);}
    }
    groups.into_iter().map(|(file,items)| {
        let entities=items.iter().map(|item|item.source.clone()).collect(); let sites=items.iter().map(|item|item.span.clone()).collect();
        super::rule(index,"unresolved_references",format!("{} unresolved references in file",items.len()),entities,vec![file],Vec::new(),sites)
    }).collect()
}
