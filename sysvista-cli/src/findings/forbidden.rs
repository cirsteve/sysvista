use std::collections::BTreeMap;
use crate::output::v2::{Finding, Snapshot};

pub fn derive(snapshot: &Snapshot) -> Vec<Finding> {
    let entity_file: BTreeMap<_,_> = snapshot.entities.iter().map(|e|(e.id.clone(),e.file_id.clone())).collect();
    let file_module: BTreeMap<_,_> = snapshot.modules.iter().flat_map(|m| m.file_ids.iter().map(move |f|(f.clone(),m.name.as_str()))).collect();
    let mut out=Vec::new();
    for relationship in &snapshot.relationships {
        let (id,source,target,_,_) = crate::validate::relationship_parts(relationship);
        let pair = entity_file.get(source).and_then(|f| file_module.get(f)).zip(entity_file.get(target).and_then(|f| file_module.get(f)));
        let Some((from,to))=pair else { continue };
        if snapshot.forbidden_dependencies.iter().any(|rule| rule.from == *from && rule.to == *to) {
            let site=snapshot.entities.iter().find(|e| &e.id==source).map(|e|e.span.clone()).into_iter().collect();
            out.push(super::rule(snapshot,"forbidden_dependency",format!("dependency {from} -> {to} is forbidden"),vec![source.clone(),target.clone()],Vec::new(),vec![id.clone()],site));
        }
    }
    out
}
