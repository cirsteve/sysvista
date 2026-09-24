use std::collections::BTreeMap;
use crate::output::v2::{Finding, Snapshot};
use super::Index;

pub fn derive(snapshot: &Snapshot, index: &Index) -> Vec<Finding> {
    let file_module: BTreeMap<_,_> = snapshot.modules.iter().flat_map(|m| m.file_ids.iter().map(move |f|(f.clone(),m.name.as_str()))).collect();
    let mut out=Vec::new();
    for relationship in &snapshot.relationships {
        let (id,source,target,_,_) = crate::validate::relationship_parts(relationship);
        let pair = index.file_of(source).and_then(|f| file_module.get(f)).zip(index.file_of(target).and_then(|f| file_module.get(f)));
        let Some((from,to))=pair else { continue };
        if snapshot.forbidden_dependencies.iter().any(|rule| rule.from == *from && rule.to == *to) {
            let site=index.span(source).into_iter().collect();
            out.push(super::rule(index,"forbidden_dependency",format!("dependency {from} -> {to} is forbidden"),vec![source.clone(),target.clone()],Vec::new(),vec![id.clone()],site));
        }
    }
    out
}
