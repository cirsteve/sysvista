use std::collections::BTreeMap;
use crate::output::v2::{self, Diagnostic, Snapshot};

pub fn validate(snapshot: &Snapshot) -> Vec<Diagnostic> {
    let mut groups: BTreeMap<(String, String), (Vec<_>, Vec<_>)> = BTreeMap::new();
    for rel in &snapshot.relationships {
        let (id, source, target, kind, origin) = super::relationship_parts(rel);
        let pair = groups.entry((source.0.clone(), kind.into())).or_default();
        if origin == "resolved" { pair.0.push((id.clone(), target.clone())); }
        if origin == "heuristic" { pair.1.push((id.clone(), target.clone())); }
    }
    let mut out = Vec::new();
    for ((_source, _kind), (resolved, heuristic)) in groups {
        for (resolved_id, resolved_target) in &resolved {
            for (heuristic_id, heuristic_target) in &heuristic {
                if resolved_target != heuristic_target {
                    let mut ids = vec![resolved_id.clone(), heuristic_id.clone()]; ids.sort();
                    out.push(Diagnostic::Contradiction { id: v2::stable_id("diagnostic", &["contradiction", ids[0].as_ref(), ids[1].as_ref()]),
                        relationship_ids: ids, message: "heuristic and resolved relationships disagree; both were retained".into() });
                }
            }
        }
    }
    out
}
