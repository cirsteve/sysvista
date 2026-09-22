use std::{fs, io, path::Path};

use serde::Serialize;

use super::{ScopeIndex, Snapshot};

#[derive(Serialize)]
struct Graph<'a> {
    source_files: &'a [super::SourceFile],
    entities: &'a [super::CodeEntity],
    modules: &'a [super::LogicalModule],
    relationships: &'a [super::Relationship],
    unresolved_references: &'a [super::UnresolvedReference],
    evidence: &'a [super::Evidence],
    claims: &'a [super::Claim],
    payload_contracts: &'a [super::PayloadContract],
    projections: &'a [super::Projection],
    findings: &'a [super::Finding],
}

pub fn write_bundle(snapshot: &Snapshot, output: &Path) -> io::Result<()> {
    let mut snapshot = snapshot.clone();
    canonicalize(&mut snapshot);
    fs::create_dir_all(output.join("index"))?;
    write_json(&output.join("manifest.json"), &snapshot.manifest)?;
    write_json(
        &output.join("graph.json"),
        &Graph {
            source_files: &snapshot.source_files,
            entities: &snapshot.entities,
            modules: &snapshot.modules,
            relationships: &snapshot.relationships,
            unresolved_references: &snapshot.unresolved_references,
            evidence: &snapshot.evidence,
            claims: &snapshot.claims,
            payload_contracts: &snapshot.payload_contracts,
            projections: &snapshot.projections,
            findings: &snapshot.findings,
        },
    )?;
    write_json(&output.join("diagnostics.json"), &snapshot.diagnostics)?;
    write_json(
        &output.join("index/scopes.json"),
        &ScopeIndex::from_snapshot(&snapshot),
    )
}

fn write_json(path: &Path, value: &impl Serialize) -> io::Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
    bytes.push(b'\n');
    fs::write(path, bytes)
}

fn canonicalize(snapshot: &mut Snapshot) {
    snapshot.source_files.sort_by(|a, b| a.id.cmp(&b.id));
    snapshot.entities.sort_by(|a, b| a.id.cmp(&b.id));
    snapshot.modules.sort_by(|a, b| a.id.cmp(&b.id));
    for module in &mut snapshot.modules {
        module.file_ids.sort();
        module.entity_ids.sort();
    }
    snapshot.relationships.sort_by(|a, b| {
        let (_, asource, atarget, akind, aorigin) = a.sort_key();
        let (_, bsource, btarget, bkind, borigin) = b.sort_key();
        (asource, atarget, akind, aorigin).cmp(&(bsource, btarget, bkind, borigin))
    });
    snapshot.unresolved_references.sort_by(|a, b| {
        (&a.source, &a.name, a.span.start_line, a.span.start_column).cmp(&(
            &b.source,
            &b.name,
            b.span.start_line,
            b.span.start_column,
        ))
    });
    snapshot
        .evidence
        .sort_by_key(|item| serde_json::to_string(item).unwrap_or_default());
    snapshot.claims.sort_by(|a, b| a.id.cmp(&b.id));
    for claim in &mut snapshot.claims {
        claim.evidence_ids.sort();
    }
    snapshot
        .payload_contracts
        .sort_by(|a, b| a.name.cmp(&b.name));
    for contract in &mut snapshot.payload_contracts {
        contract.producer_ids.sort();
        contract.consumer_ids.sort();
    }
    snapshot
        .diagnostics
        .sort_by_key(|item| serde_json::to_string(item).unwrap_or_default());
    snapshot.projections.sort_by(|a, b| a.id.cmp(&b.id));
    for projection in &mut snapshot.projections {
        projection.entity_ids.sort();
        projection.relationship_ids.sort();
    }
    snapshot
        .findings
        .sort_by_key(|item| serde_json::to_string(item).unwrap_or_default());
    snapshot
        .manifest
        .inventory_entries
        .sort_by(|a, b| a.path.cmp(&b.path));
}
