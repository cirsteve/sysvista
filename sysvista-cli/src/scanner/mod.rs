pub mod file_walker;
pub mod language;
pub mod models;
pub mod prompts;
pub mod relationships;
pub mod services;
pub mod transforms;
pub mod transports;
pub mod workflows;

use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::io;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

#[cfg(test)]
use crate::output::schema::ComponentKind;
use crate::output::schema::{DetectedComponent, ScanStats, SysVistaOutput};
use crate::{
    discovery::{Config, Inventory, InventoryOutcome},
    output::v2::{
        self, AnalysisStatus, Diagnostic, InventoryCounts, Manifest, Snapshot, SourceFile,
    },
};

/// Create a deterministic ID from kind + name + file
pub fn make_id(kind: &str, name: &str, file: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{kind}:{name}:{file}"));
    let result = hasher.finalize();
    format!("{:x}", result)[..16].to_string()
}

pub fn scan(root: &Path) -> SysVistaOutput {
    let start = Instant::now();

    let (files, files_skipped) = file_walker::walk_directory(root);

    let mut all_components: Vec<DetectedComponent> = Vec::new();
    let mut languages_seen: HashSet<String> = HashSet::new();
    let mut file_contents: HashMap<String, String> = HashMap::new();
    let mut files_scanned: u64 = 0;

    for walked in &files {
        let lang = match language::detect_language(&walked.path) {
            Some(l) => l,
            None => continue,
        };

        let content = match std::fs::read_to_string(&walked.path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        files_scanned += 1;
        languages_seen.insert(lang.to_string());
        file_contents.insert(walked.relative_path.clone(), content.clone());

        // Detect components
        let mut components = Vec::new();
        components.extend(models::detect_models(&content, lang, &walked.relative_path));
        components.extend(services::detect_services(
            &content,
            lang,
            &walked.relative_path,
        ));
        components.extend(transports::detect_transports(
            &content,
            lang,
            &walked.relative_path,
        ));
        components.extend(transforms::detect_transforms(
            &content,
            lang,
            &walked.relative_path,
        ));
        components.extend(prompts::detect_prompts(
            &content,
            lang,
            &walked.relative_path,
        ));

        all_components.extend(components);
    }

    // Deduplicate components by ID (multiple patterns can match the same definition)
    let mut seen_ids = HashSet::new();
    all_components.retain(|c| seen_ids.insert(c.id.clone()));

    // Infer edges
    let mut edges = relationships::infer_edges(&all_components, &file_contents);

    // Merge flow edges (handles, persists, transforms, consumes, produces).
    // These carry semantic meaning for the flow view even when an import/reference
    // edge already exists for the same pair.
    edges.extend(relationships::infer_flow_edges(
        &all_components,
        &file_contents,
    ));

    // Merge call/dispatch edges.
    edges.extend(relationships::infer_call_edges(
        &all_components,
        &file_contents,
    ));

    // Infer workflows from components and edges
    let workflows = workflows::infer_workflows(&all_components, &edges);

    let duration = start.elapsed();

    let project_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut detected_languages: Vec<String> = languages_seen.into_iter().collect();
    detected_languages.sort();

    SysVistaOutput {
        version: "1".to_string(),
        scanned_at: chrono::Utc::now().to_rfc3339(),
        root_dir: root.to_string_lossy().to_string(),
        project_name,
        detected_languages,
        components: all_components,
        edges,
        workflows,
        scan_stats: ScanStats {
            files_scanned,
            files_skipped,
            scan_duration_ms: duration.as_millis() as u64,
        },
    }
}

pub fn scan_v2(root: &Path, config: &Config) -> io::Result<Snapshot> {
    let (repository, portable) = repository_identity(root, config);
    let mut inventory = Inventory::discover(root, config)?;
    let mut source_files = Vec::new();
    let mut diagnostics = Vec::new();

    if !portable {
        diagnostics.push(Diagnostic::Warning {
            message: "repository identity fell back to the directory basename; IDs are not portable across renamed checkouts".into(),
            span: None,
        });
    }

    for entry in &mut inventory.entries {
        let path = Inventory::absolute_path(root, entry);
        let id = v2::file_id(&repository, &entry.path);
        match &entry.outcome {
            InventoryOutcome::Excluded { .. } => continue,
            InventoryOutcome::Unreadable { io_error } => {
                let diagnostic_id =
                    v2::stable_id("diagnostic", &["unreadable", &entry.path, io_error]);
                diagnostics.push(Diagnostic::UnreadableFile {
                    id: diagnostic_id,
                    path: entry.path.clone(),
                    message: io_error.clone(),
                });
                source_files.push(SourceFile {
                    id,
                    path: entry.path.clone(),
                    language: None,
                    analysis: AnalysisStatus::Failed {
                        message: io_error.clone(),
                    },
                    content_hash: None, byte_length: None, line_count: None,
                });
            }
            InventoryOutcome::Failed { diagnostic_id } => {
                diagnostics.push(Diagnostic::FailedFile {
                    id: diagnostic_id.clone(),
                    path: entry.path.clone(),
                    message: "path discovery failed".into(),
                });
                source_files.push(SourceFile {
                    id,
                    path: entry.path.clone(),
                    language: None,
                    analysis: AnalysisStatus::Failed {
                        message: "path discovery failed".into(),
                    },
                    content_hash: None, byte_length: None, line_count: None,
                });
            }
            InventoryOutcome::Unsupported => {
                source_files.push(SourceFile {
                    id,
                    path: entry.path.clone(),
                    language: None,
                    analysis: AnalysisStatus::Unsupported,
                    content_hash: None, byte_length: None, line_count: None,
                });
            }
            InventoryOutcome::Included => {
                let language = language::detect_language_with_config(&path, config)
                    .unwrap_or("unknown")
                    .to_owned();
                match std::fs::read_to_string(&path) {
                    Ok(_) => source_files.push(SourceFile {
                        id,
                        path: entry.path.clone(),
                        language: Some(language.clone()),
                        analysis: if language == "typescript" || language == "javascript" {
                            AnalysisStatus::None
                        } else {
                            AnalysisStatus::Parsed {
                                analyzer: "builtin-heuristic".into(),
                            }
                        },
                        content_hash: None, byte_length: None, line_count: None,
                    }),
                    Err(error) => {
                        let message = error.to_string();
                        entry.outcome = InventoryOutcome::Unreadable {
                            io_error: message.clone(),
                        };
                        let diagnostic_id =
                            v2::stable_id("diagnostic", &["unreadable", &entry.path, &message]);
                        diagnostics.push(Diagnostic::UnreadableFile {
                            id: diagnostic_id,
                            path: entry.path.clone(),
                            message: message.clone(),
                        });
                        source_files.push(SourceFile {
                            id,
                            path: entry.path.clone(),
                            language: Some(language),
                            analysis: AnalysisStatus::Failed { message },
                            content_hash: None, byte_length: None, line_count: None,
                        });
                    }
                }
            }
        }
    }
    for file in &mut source_files {
        if let Ok(bytes) = std::fs::read(root.join(&file.path)) {
            file.content_hash = Some(format!("{:x}", Sha256::digest(&bytes)));
            file.byte_length = Some(bytes.len() as u64);
            file.line_count = Some(bytes.split(|byte| *byte == b'\n').count().max(1) as u32);
        }
    }

    // The legacy detectors are exposed as one explicit stage. Keep the source-file
    // accounting above, but use the stage output for all v2 graph values.
    let heuristic = crate::heuristic::analyze(root, &repository, &inventory, config);
    let analyzer_files: Vec<_> = source_files
        .iter()
        .filter(|file| matches!(file.language.as_deref(), Some("typescript" | "javascript")))
        .map(|file| file.path.clone())
        .collect();
    let mut analyzer_versions = std::collections::BTreeMap::new();
    let mut analyzer_response = None;
    if !analyzer_files.is_empty() {
        let request = crate::analyzer::AnalyzeRequest {
            contract_version: crate::analyzer::CONTRACT_VERSION,
            root: root.display().to_string(),
            files: analyzer_files,
            tsconfig: None,
        };
        match crate::analyzer::analyze(&request) {
            Ok(response) => {
                mark_typescript_analysis(
                    &mut source_files,
                    AnalysisStatus::Parsed {
                        analyzer: "typescript-compiler-api".into(),
                    },
                );
                analyzer_versions.insert("typescript".into(), response.analyzer_version.clone());
                analyzer_response = Some(response);
            }
            Err(crate::analyzer::AnalyzerError::ContractMismatch { expected, actual }) => {
                let message =
                    format!("analyzer contract mismatch: expected {expected}, got {actual}");
                mark_typescript_analysis(
                    &mut source_files,
                    AnalysisStatus::Failed {
                        message: message.clone(),
                    },
                );
                diagnostics.push(Diagnostic::AnalyzerContractMismatch {
                    message,
                    severity: "error".into(),
                });
            }
            Err(error) => {
                let message = error.to_string();
                mark_typescript_analysis(
                    &mut source_files,
                    AnalysisStatus::Failed {
                        message: message.clone(),
                    },
                );
                diagnostics.push(Diagnostic::AnalyzerUnavailable {
                    message,
                    severity: "error".into(),
                });
            }
        }
    }
    let response = analyzer_response.unwrap_or(crate::analyzer::AnalyzeResponse {
        contract_version: crate::analyzer::CONTRACT_VERSION,
        analyzer_version: "unavailable".into(),
        entities: Vec::new(),
        relationships: Vec::new(),
        unresolved: Vec::new(),
        diagnostics: Vec::new(),
        payloads: Vec::new(),
    });
    let inventoried: std::collections::BTreeSet<String> =
        source_files.iter().map(|file| file.path.clone()).collect();
    let root_text = root.display().to_string();
    let merged = crate::analyzer::merge(
        &crate::analyzer::MergeContext {
            repository: &repository,
            root: &root_text,
            files: &inventoried,
        },
        response,
        heuristic,
    );
    diagnostics.extend(merged.diagnostics);
    let counts = inventory_counts(&inventory);
    let mut snapshot = Snapshot {
        manifest: Manifest {
            schema_version: "3".into(),
            root_scope_id: v2::ScopeId(v2::stable_id("scope", &["repository", &repository])),
            repository,
            scanned_at: chrono::Utc::now().to_rfc3339(),
            root: root.display().to_string(),
            tool_version: env!("CARGO_PKG_VERSION").into(),
            analyzer_versions,
            inventory: counts,
            inventory_entries: inventory.entries.clone(),
            validation: Default::default(),
            files: Vec::new(),
            source_included: false,
        },
        source_files,
        entities: merged.entities,
        modules: Vec::new(),
        relationships: merged.relationships,
        unresolved_references: merged.unresolved,
        evidence: merged.evidence,
        claims: merged.claims,
        payload_contracts: merged.payloads,
        diagnostics,
        projections: Vec::new(),
        findings: Vec::new(),
        forbidden_dependencies: Vec::new(),
        scope_index: None,
    };
    let duplicates = v2::dedup_records(&mut snapshot);
    snapshot.diagnostics.extend(duplicates);
    crate::hierarchy::derive(&mut snapshot, &inventory, config);
    snapshot.scope_index = Some(v2::ScopeIndex::with_extensions(&snapshot, &config.viewer.visible_extensions));
    let validation = crate::validate::validate(&snapshot);
    snapshot.manifest.validation = crate::validate::summary(&validation);
    snapshot.diagnostics.extend(validation);
    snapshot.findings = crate::findings::derive_with_exclusions(&snapshot, &config.findings.unresolved.exclude_reasons);
    Ok(snapshot)
}

fn mark_typescript_analysis(source_files: &mut [SourceFile], status: AnalysisStatus) {
    for file in source_files
        .iter_mut()
        .filter(|file| matches!(file.language.as_deref(), Some("typescript" | "javascript")))
    {
        file.analysis = status.clone();
    }
}

#[cfg(test)]
fn assign_scan_component_ids(components: &mut [DetectedComponent]) {
    for (index, component) in components.iter_mut().enumerate() {
        component.id = v2::stable_id(
            "scan_component",
            &[
                &component.source.file,
                kind_name(&component.kind),
                &component.name,
                &index.to_string(),
            ],
        );
    }
}

#[cfg(test)]
fn kind_name(kind: &ComponentKind) -> &'static str {
    match kind {
        ComponentKind::Model => "model",
        ComponentKind::Service => "service",
        ComponentKind::Transport => "transport",
        ComponentKind::Transform => "transform",
        ComponentKind::Prompt => "prompt",
    }
}

#[cfg(test)]
fn component_attributes(
    component: &DetectedComponent,
) -> std::collections::BTreeMap<String, serde_json::Value> {
    let mut attributes = std::collections::BTreeMap::new();
    for (key, value) in &component.metadata {
        attributes.insert(key.clone(), value.clone().into());
    }
    if let Some(value) = &component.prompt_subtype {
        attributes.insert("prompt_subtype".into(), value.clone().into());
    }
    if let Some(value) = &component.http_method {
        attributes.insert("http_method".into(), value.clone().into());
    }
    if let Some(value) = &component.http_path {
        attributes.insert("http_path".into(), value.clone().into());
    }
    if let Some(value) = &component.model_fields {
        attributes.insert("model_fields".into(), serde_json::json!(value));
    }
    if let Some(value) = &component.transport_protocol {
        attributes.insert(
            "transport_protocol".into(),
            serde_json::to_value(value).expect("transport protocol is serializable"),
        );
    }
    if let Some(value) = &component.consumes {
        attributes.insert("consumes".into(), serde_json::json!(value));
    }
    if let Some(value) = &component.produces {
        attributes.insert("produces".into(), serde_json::json!(value));
    }
    attributes
}

fn inventory_counts(inventory: &Inventory) -> InventoryCounts {
    let mut counts = InventoryCounts::default();
    for entry in &inventory.entries {
        match entry.outcome {
            InventoryOutcome::Included => counts.included += 1,
            InventoryOutcome::Excluded { .. } => counts.excluded += 1,
            InventoryOutcome::Unsupported => counts.unsupported += 1,
            InventoryOutcome::Unreadable { .. } => counts.unreadable += 1,
            InventoryOutcome::Failed { .. } => counts.failed += 1,
        }
    }
    counts
}

fn repository_identity(root: &Path, config: &Config) -> (String, bool) {
    if let Ok(output) = Command::new("git")
        .args(["-C"])
        .arg(root)
        .args(["config", "--get", "remote.origin.url"])
        .output()
    {
        if output.status.success() {
            let remote = String::from_utf8_lossy(&output.stdout);
            let normalized = normalize_remote(remote.trim());
            if !normalized.is_empty() {
                return (normalized, true);
            }
        }
    }
    if let Some(name) = config
        .repository
        .name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
    {
        return (name.trim().to_owned(), true);
    }
    (
        root.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_owned(),
        false,
    )
}

fn normalize_remote(remote: &str) -> String {
    let mut value = remote
        .trim()
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .to_owned();
    if let Some((_, rest)) = value.split_once("://") {
        value = rest.to_owned();
    }
    if value.starts_with("git@") {
        value = value.trim_start_matches("git@").replacen(':', "/", 1);
    }
    if let Some((_, rest)) = value.split_once('@') {
        value = rest.to_owned();
    }
    value.trim_start_matches('/').to_owned()
}

#[cfg(test)]
mod v2_conversion_tests {
    use std::collections::HashMap;

    use super::*;
    use crate::output::schema::{SourceLocation, TransportProtocol};

    fn component(line: u32) -> DetectedComponent {
        DetectedComponent {
            id: "legacy-collision".into(),
            name: "Widget".into(),
            kind: ComponentKind::Transport,
            language: "typescript".into(),
            source: SourceLocation {
                file: "src/widget.ts".into(),
                line_start: Some(line),
                line_end: None,
            },
            metadata: HashMap::from([("detection".into(), "fixture".into())]),
            transport_protocol: Some(TransportProtocol::Http),
            http_method: Some("POST".into()),
            http_path: Some("/widgets".into()),
            model_fields: Some(vec!["name".into()]),
            prompt_subtype: Some("generator".into()),
            consumes: Some(vec!["WidgetInput".into()]),
            produces: Some(vec!["Widget".into()]),
        }
    }

    #[test]
    fn scan_component_ids_disambiguate_legacy_collisions_without_using_lines() {
        let mut original = vec![component(10), component(20)];
        let mut shifted = vec![component(11), component(21)];
        assign_scan_component_ids(&mut original);
        assign_scan_component_ids(&mut shifted);
        assert_ne!(original[0].id, original[1].id);
        assert_eq!(original[0].id, shifted[0].id);
        assert_eq!(original[1].id, shifted[1].id);
    }

    #[test]
    fn component_attributes_preserve_detector_metadata() {
        let attributes = component_attributes(&component(10));
        for key in [
            "detection",
            "transport_protocol",
            "http_method",
            "http_path",
            "model_fields",
            "prompt_subtype",
            "consumes",
            "produces",
        ] {
            assert!(attributes.contains_key(key), "missing {key}");
        }
        assert_eq!(attributes["transport_protocol"], "http");
    }
}
