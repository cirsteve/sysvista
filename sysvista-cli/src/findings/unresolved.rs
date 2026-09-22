use std::collections::BTreeMap;
use crate::output::v2::{Finding, Snapshot};

pub fn derive(snapshot: &Snapshot) -> Vec<Finding> {
    let entity_file:BTreeMap<_,_>=snapshot.entities.iter().map(|e|(e.id.clone(),e.file_id.clone())).collect();
    let mut groups:BTreeMap<_,Vec<_>>=BTreeMap::new();
    for unresolved in &snapshot.unresolved_references { if let Some(file)=entity_file.get(&unresolved.source){groups.entry(file.clone()).or_default().push(unresolved);} }
    groups.into_iter().map(|(file,items)| {
        let entities=items.iter().map(|item|item.source.clone()).collect(); let sites=items.iter().map(|item|item.span.clone()).collect();
        super::rule(snapshot,"unresolved_references",format!("{} unresolved references in file",items.len()),entities,vec![file],Vec::new(),sites)
    }).collect()
}
