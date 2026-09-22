mod diagnostics;
mod entities;
mod ids;
mod index;
mod projection;
mod relationships;
mod writer;

use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

pub use diagnostics::{Diagnostic, Finding};
#[allow(unused_imports)]
pub use entities::{AnalysisStatus, CodeEntity, LogicalModule, SourceFile, SourceSpan};
pub use ids::{
    EntityId, FileId, ModuleId, RelationshipId, ScopeId, entity_id, file_id, relationship_id,
    scope_id, stable_id,
};
pub use index::ScopeIndex;
pub use projection::Projection;
pub use relationships::{Claim, Evidence, PayloadContract, Relationship, UnresolvedReference};
pub use writer::write_bundle;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct InventoryCounts {
    pub included: u64,
    pub excluded: u64,
    pub unsupported: u64,
    pub unreadable: u64,
    pub failed: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Manifest {
    pub schema_version: String,
    pub repository: String,
    pub scanned_at: String,
    pub root: String,
    pub tool_version: String,
    #[serde(default)]
    pub analyzer_versions: BTreeMap<String, String>,
    pub inventory: InventoryCounts,
    #[serde(default)]
    pub inventory_entries: Vec<crate::discovery::InventoryEntry>,
    #[serde(default)]
    pub validation: ValidationSummary,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub source_included: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ValidationSummary {
    pub diagnostics: u64,
    pub errors: u64,
    pub warnings: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Snapshot {
    pub manifest: Manifest,
    #[serde(default)]
    pub source_files: Vec<SourceFile>,
    #[serde(default)]
    pub entities: Vec<CodeEntity>,
    #[serde(default)]
    pub modules: Vec<LogicalModule>,
    #[serde(default)]
    pub relationships: Vec<Relationship>,
    #[serde(default)]
    pub unresolved_references: Vec<UnresolvedReference>,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    #[serde(default)]
    pub claims: Vec<Claim>,
    #[serde(default)]
    pub payload_contracts: Vec<PayloadContract>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    #[serde(default)]
    pub projections: Vec<Projection>,
    #[serde(default)]
    pub findings: Vec<Finding>,
}

pub fn default_schema_path() -> PathBuf {
    PathBuf::from("schema/sysvista-v2.schema.json")
}

pub fn write_schema(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let schema = schema_for!(Snapshot);
    let mut bytes = serde_json::to_vec_pretty(&schema).map_err(io::Error::other)?;
    bytes.push(b'\n');
    fs::write(path, bytes)
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn variant_types_are_tagged_unions() {
        let schema = serde_json::to_value(schema_for!(Snapshot)).unwrap();
        for name in ["Relationship", "Evidence", "Diagnostic", "Finding"] {
            let definition = &schema["$defs"][name];
            let variants = definition["oneOf"]
                .as_array()
                .unwrap_or_else(|| panic!("{name} must be represented with oneOf"));
            assert!(!variants.is_empty());
            for variant in variants {
                assert_literal_kind(variant, name);
            }
        }
    }

    #[test]
    fn schema_default_is_relative_to_runtime_directory() {
        assert_eq!(
            default_schema_path(),
            PathBuf::from("schema/sysvista-v2.schema.json")
        );
    }

    fn assert_literal_kind(variant: &Value, name: &str) {
        let kind = &variant["properties"]["kind"];
        assert!(
            kind.get("const").is_some()
                || kind
                    .get("enum")
                    .and_then(Value::as_array)
                    .is_some_and(|v| v.len() == 1),
            "{name} variant lacks a literal kind tag: {kind}"
        );
        assert!(
            variant["required"]
                .as_array()
                .is_some_and(|required| required.iter().any(|v| v == "kind")),
            "{name} variant does not require kind"
        );
    }
}
