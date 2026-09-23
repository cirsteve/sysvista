use std::collections::{BTreeMap, BTreeSet};
use crate::output::v2::{FileId, Finding, Relationship, Snapshot};
use super::Index;

pub fn derive(snapshot: &Snapshot, index: &Index) -> Vec<Finding> {
    let mut graph: BTreeMap<_, BTreeSet<_>> = snapshot.source_files.iter().map(|f| (f.id.clone(), BTreeSet::new())).collect();
    let mut edges = Vec::new();
    for relationship in &snapshot.relationships {
        if let Relationship::Imports { id, source, target, origin, .. } = relationship {
            if origin != "resolved" { continue; }
            let (Some(from), Some(to)) = (index.file_of(source), index.file_of(target)) else { continue };
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
        let sites = used.iter().filter_map(|(_,source,_,_,_)| index.span(source)).collect();
        findings.push(super::rule(index, "import_cycle", format!("resolved import cycle contains {} files", files.len()), entities, files, relationships, sites));
    }
    findings
}

/// Kosaraju's algorithm with explicit stacks: a long import chain must not overflow
/// the thread stack, which `catch_unwind` cannot recover from.
pub(crate) fn strongly_connected(graph: &BTreeMap<FileId, BTreeSet<FileId>>) -> Vec<Vec<FileId>> {
    let empty = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut order = Vec::new();
    for start in graph.keys() {
        if !seen.insert(start) { continue; }
        // Each frame is a node and the iterator over its remaining successors.
        let mut stack = vec![(start, graph.get(start).unwrap_or(&empty).iter())];
        while let Some((node, successors)) = stack.last_mut() {
            if let Some(next) = successors.next() {
                if seen.insert(next) { stack.push((next, graph.get(next).unwrap_or(&empty).iter())); }
            } else {
                order.push(*node);
                stack.pop();
            }
        }
    }
    let mut reverse: BTreeMap<&FileId, Vec<&FileId>> = graph.keys().map(|id| (id, Vec::new())).collect();
    for (from, next) in graph { for to in next { reverse.entry(to).or_default().push(from); } }
    let mut assigned = BTreeSet::new();
    let mut out = Vec::new();
    while let Some(root) = order.pop() {
        if !assigned.insert(root) { continue; }
        let mut component = Vec::new();
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            component.push(node.clone());
            for &previous in reverse.get(node).into_iter().flatten() {
                if assigned.insert(previous) { pending.push(previous); }
            }
        }
        out.push(component);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: usize) -> FileId { FileId(format!("file:{value:06}")) }

    #[test]
    fn long_chains_do_not_recurse_and_cycles_are_found() {
        // A 200k-file chain would overflow a recursive traversal.
        let length = 200_000;
        let mut graph: BTreeMap<_, BTreeSet<_>> = (0..length).map(|i| (id(i), BTreeSet::from([id(i + 1)]))).collect();
        graph.insert(id(length), BTreeSet::new());
        let components = strongly_connected(&graph);
        assert_eq!(components.len(), length + 1);
        assert!(components.iter().all(|c| c.len() == 1));

        graph.get_mut(&id(length)).unwrap().insert(id(length - 2));
        let cyclic: Vec<_> = strongly_connected(&graph).into_iter().filter(|c| c.len() > 1).collect();
        assert_eq!(cyclic.len(), 1);
        let mut members = cyclic[0].clone();
        members.sort();
        assert_eq!(members, vec![id(length - 2), id(length - 1), id(length)]);
    }
}
