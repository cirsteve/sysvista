use std::collections::{BTreeMap, BTreeSet};

use super::{Diagnostic, Evidence, Finding, Snapshot};

/// Ids that occur more than once in one collection, keyed by collection name.
/// Consumers index every collection by id, so any entry here loses records.
pub fn duplicate_ids(snapshot: &Snapshot) -> BTreeMap<&'static str, Vec<String>> {
    let mut out = BTreeMap::new();
    let mut check = |name: &'static str, ids: Vec<&str>| {
        let mut seen = BTreeSet::new();
        let duplicates: BTreeSet<_> = ids.into_iter().filter(|id| !seen.insert(*id)).map(str::to_owned).collect();
        if !duplicates.is_empty() {
            out.insert(name, duplicates.into_iter().collect());
        }
    };
    check("source_files", snapshot.source_files.iter().map(|f| f.id.as_ref()).collect());
    check("entities", snapshot.entities.iter().map(|e| e.id.as_ref()).collect());
    check("modules", snapshot.modules.iter().map(|m| m.id.as_ref()).collect());
    check("relationships", snapshot.relationships.iter().map(|r| r.sort_key().0.as_ref()).collect());
    check("evidence", snapshot.evidence.iter().map(evidence_id).collect());
    check("claims", snapshot.claims.iter().map(|c| c.id.as_str()).collect());
    check("projections", snapshot.projections.iter().map(|p| p.id.as_str()).collect());
    check(
        "findings",
        snapshot
            .findings
            .iter()
            .filter_map(|finding| match finding {
                Finding::Rule { id, .. } => Some(id.as_str()),
                _ => None,
            })
            .collect(),
    );
    out
}

/// Collapse records that are exact duplicates, then report any id still shared by
/// records with different content. The first record for such an id is kept.
pub fn dedup_records(snapshot: &mut Snapshot) -> Vec<Diagnostic> {
    snapshot.evidence.sort_by_key(|v| serde_json::to_string(v).unwrap_or_default());
    snapshot.evidence.dedup();
    snapshot.claims.sort_by_key(|v| serde_json::to_string(v).unwrap_or_default());
    snapshot.claims.dedup();
    let mut diagnostics = Vec::new();
    for (collection, ids) in duplicate_ids(snapshot) {
        for id in ids {
            diagnostics.push(Diagnostic::Warning {
                message: format!("{collection} id {id} was emitted for different records; only the first was kept"),
                span: None,
            });
        }
    }
    keep_first(&mut snapshot.source_files, |f| Some(f.id.as_ref()));
    keep_first(&mut snapshot.entities, |e| Some(e.id.as_ref()));
    keep_first(&mut snapshot.modules, |m| Some(m.id.as_ref()));
    keep_first(&mut snapshot.relationships, |r| Some(r.sort_key().0.as_ref()));
    keep_first(&mut snapshot.evidence, |e| Some(evidence_id(e)));
    keep_first(&mut snapshot.claims, |c| Some(c.id.as_str()));
    keep_first(&mut snapshot.projections, |p| Some(p.id.as_str()));
    keep_first(&mut snapshot.findings, |finding| match finding {
        Finding::Rule { id, .. } => Some(id.as_str()),
        _ => None,
    });
    diagnostics
}

/// Keep the first record for each id; records without an id are all kept.
fn keep_first<T>(items: &mut Vec<T>, id: impl Fn(&T) -> Option<&str>) {
    let mut seen = BTreeSet::new();
    items.retain(|item| id(item).is_none_or(|id| seen.insert(id.to_owned())));
}

fn evidence_id(evidence: &Evidence) -> &str {
    match evidence {
        Evidence::Source { id, .. }
        | Evidence::SourceSnapshot { id, .. }
        | Evidence::Text { id, .. }
        | Evidence::Analyzer { id, .. } => id,
    }
}
