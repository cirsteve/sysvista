use std::collections::{BTreeMap, BTreeSet};
use crate::output::v2::{Finding, Relationship, Snapshot};

pub fn derive(snapshot: &Snapshot) -> Vec<Finding> {
    let entity_file: BTreeMap<_,_> = snapshot.entities.iter().map(|e| (e.id.clone(), e.file_id.clone())).collect();
    let mut graph: BTreeMap<_, BTreeSet<_>> = snapshot.source_files.iter().map(|f| (f.id.clone(), BTreeSet::new())).collect();
    let mut edges = Vec::new();
    for relationship in &snapshot.relationships {
        if let Relationship::Imports { id, source, target, origin, .. } = relationship {
            if origin != "resolved" { continue; }
            let (Some(from), Some(to)) = (entity_file.get(source), entity_file.get(target)) else { continue };
            graph.entry(from.clone()).or_default().insert(to.clone());
            edges.push((id.clone(), source.clone(), target.clone(), from.clone(), to.clone()));
        }
    }
    let components = strongly_connected(&graph);
    let mut findings = Vec::new();
    for mut files in components.into_iter().filter(|c| c.len() > 1 || c.first().is_some_and(|f| graph.get(f).is_some_and(|n| n.contains(f)))) {
        files.sort();
        let members: BTreeSet<_> = files.iter().cloned().collect();
        let used: Vec<_> = edges.iter().filter(|(_,_,_,from,to)| members.contains(from) && members.contains(to)).collect();
        let entities = used.iter().flat_map(|(_,s,t,_,_)| [s.clone(),t.clone()]).collect();
        let relationships = used.iter().map(|(id,_,_,_,_)| id.clone()).collect();
        let sites = used.iter().filter_map(|(_,source,_,_,_)| snapshot.entities.iter().find(|e| &e.id == source).map(|e| e.span.clone())).collect();
        findings.push(super::rule(snapshot, "import_cycle", format!("resolved import cycle contains {} files", files.len()), entities, files, relationships, sites));
    }
    findings
}

fn strongly_connected(graph: &BTreeMap<crate::output::v2::FileId, BTreeSet<crate::output::v2::FileId>>) -> Vec<Vec<crate::output::v2::FileId>> {
    type Id = crate::output::v2::FileId;
    fn visit(node: &Id, graph: &BTreeMap<Id,BTreeSet<Id>>, seen: &mut BTreeSet<Id>, order: &mut Vec<Id>) {
        if !seen.insert(node.clone()) { return; }
        if let Some(next) = graph.get(node) { for child in next { visit(child, graph, seen, order); } }
        order.push(node.clone());
    }
    fn collect(node: &Id, graph: &BTreeMap<Id,BTreeSet<Id>>, seen: &mut BTreeSet<Id>, component: &mut Vec<Id>) {
        if !seen.insert(node.clone()) { return; } component.push(node.clone());
        if let Some(next) = graph.get(node) { for child in next { collect(child, graph, seen, component); } }
    }
    let mut seen=BTreeSet::new(); let mut order=Vec::new();
    for node in graph.keys() { visit(node,graph,&mut seen,&mut order); }
    let mut reverse: BTreeMap<Id,BTreeSet<Id>> = graph.keys().map(|id|(id.clone(),BTreeSet::new())).collect();
    for (from,next) in graph { for to in next { reverse.entry(to.clone()).or_default().insert(from.clone()); } }
    seen.clear(); let mut out=Vec::new();
    while let Some(node)=order.pop() { if seen.contains(&node) { continue; } let mut component=Vec::new(); collect(&node,&reverse,&mut seen,&mut component); out.push(component); }
    out
}
