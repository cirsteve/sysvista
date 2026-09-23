pub mod archive;
pub mod layout;
pub mod source_index;

use crate::output::v2::{self, ScopeIndex, Snapshot};
pub use archive::{ArchiveOptions, write_archive};
pub use layout::{MAX_ARCHIVE_BYTES, MAX_ENTRY_BYTES, validate_entry_path};
use serde::Serialize;
pub use source_index::{SourceIndex, SourceIndexEntry};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum BundleError {
    Io(io::Error),
    Zip(zip::result::ZipError),
    UnsafePath(PathBuf),
    EntryTooLarge { path: String, bytes: u64, cap: u64 },
    ArchiveTooLarge { bytes: u64, cap: u64 },
    InvalidBundle(String),
}
pub type BundleResult<T> = Result<T, BundleError>;
impl fmt::Display for BundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Zip(e) => write!(f, "{e}"),
            Self::UnsafePath(p) => write!(f, "unsafe bundle path: {}", p.display()),
            Self::EntryTooLarge { path, bytes, cap } => {
                write!(f, "bundle entry {path} is {bytes} bytes (cap {cap})")
            }
            Self::ArchiveTooLarge { bytes, cap } => {
                write!(f, "archive is {bytes} bytes (cap {cap})")
            }
            Self::InvalidBundle(m) => write!(f, "invalid bundle: {m}"),
        }
    }
}
impl std::error::Error for BundleError {}
impl From<io::Error> for BundleError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<zip::result::ZipError> for BundleError {
    fn from(value: zip::result::ZipError) -> Self {
        Self::Zip(value)
    }
}

#[derive(Serialize)]
struct Graph<'a> {
    source_files: &'a [v2::SourceFile],
    entities: &'a [v2::CodeEntity],
    modules: &'a [v2::LogicalModule],
    relationships: &'a [v2::Relationship],
    unresolved_references: &'a [v2::UnresolvedReference],
    evidence: &'a [v2::Evidence],
    claims: &'a [v2::Claim],
    payload_contracts: &'a [v2::PayloadContract],
    projections: &'a [v2::Projection],
    findings: &'a [v2::Finding],
    forbidden_dependencies: &'a [v2::ForbiddenDependencyRule],
}

pub fn write_directory(snapshot: &Snapshot, output: &Path) -> BundleResult<()> {
    let mut snapshot = snapshot.clone();
    canonicalize(&mut snapshot);
    let mut source_diagnostics = Vec::new();
    let source_index =
        SourceIndex::from_snapshot(&snapshot, MAX_ENTRY_BYTES, &mut source_diagnostics);
    snapshot.diagnostics.extend(source_diagnostics);
    snapshot
        .diagnostics
        .sort_by_key(|value| serde_json::to_string(value).unwrap_or_default());
    snapshot.manifest.validation = crate::validate::summary(&snapshot.diagnostics);
    snapshot.manifest.files = vec![
        "diagnostics.json".into(),
        "findings.json".into(),
        "graph.json".into(),
        "index/scopes.json".into(),
        "manifest.json".into(),
        "source-index.json".into(),
    ];
    fs::create_dir_all(output.join("index"))?;
    write_json(output, "manifest.json", &snapshot.manifest)?;
    write_json(
        output,
        "graph.json",
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
            forbidden_dependencies: &snapshot.forbidden_dependencies,
        },
    )?;
    write_json(output, "diagnostics.json", &snapshot.diagnostics)?;
    write_json(output, "findings.json", &snapshot.findings)?;
    write_json(output, "source-index.json", &source_index)?;
    write_json(
        output,
        "index/scopes.json",
        &ScopeIndex::from_snapshot(&snapshot),
    )?;
    Ok(())
}

pub fn write_bytes(root: &Path, entry: &str, bytes: &[u8]) -> BundleResult<()> {
    let relative = validate_entry_path(entry)?;
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    Ok(())
}
fn write_json(root: &Path, entry: &str, value: &impl Serialize) -> BundleResult<()> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
    bytes.push(b'\n');
    write_bytes(root, entry, &bytes)
}
pub(crate) fn canonicalize(snapshot: &mut Snapshot) {
    snapshot.source_files.sort_by(|a, b| a.id.cmp(&b.id));
    snapshot.entities.sort_by(|a, b| a.id.cmp(&b.id));
    snapshot.modules.sort_by(|a, b| a.id.cmp(&b.id));
    for module in &mut snapshot.modules {
        module.file_ids.sort();
        module.entity_ids.sort();
    }
    snapshot
        .relationships
        .sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
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
        .sort_by_key(|v| serde_json::to_string(v).unwrap_or_default());
    snapshot.evidence.dedup();
    for claim in &mut snapshot.claims {
        claim.evidence_ids.sort();
        claim.evidence_ids.dedup();
    }
    snapshot.claims.sort_by_key(|v| (v.id.clone(), serde_json::to_string(v).unwrap_or_default()));
    snapshot.claims.dedup();
    snapshot
        .payload_contracts
        .sort_by(|a, b| a.name.cmp(&b.name));
    for contract in &mut snapshot.payload_contracts {
        contract.producer_ids.sort();
        contract.consumer_ids.sort();
    }
    snapshot
        .diagnostics
        .sort_by_key(|v| serde_json::to_string(v).unwrap_or_default());
    snapshot.projections.sort_by(|a, b| a.id.cmp(&b.id));
    for projection in &mut snapshot.projections {
        projection.entity_ids.sort();
        projection.relationship_ids.sort();
    }
    snapshot
        .manifest
        .inventory_entries
        .sort_by_key(|entry| serde_json::to_string(entry).unwrap_or_default());
    snapshot
        .findings
        .sort_by_key(|v| serde_json::to_string(v).unwrap_or_default());
}
