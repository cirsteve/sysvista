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

use crate::output::schema::{DetectedComponent, ScanStats, SysVistaOutput};
use crate::{
    discovery::{Config, Inventory, InventoryOutcome},
    output::{
        schema::ComponentKind,
        v2::{
            self, AnalysisStatus, CodeEntity, Diagnostic, InventoryCounts, Manifest, Relationship,
            Snapshot, SourceFile, SourceSpan,
        },
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
    let mut components = Vec::new();
    let mut file_contents = HashMap::new();
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
                });
            }
            InventoryOutcome::Unsupported => {
                source_files.push(SourceFile {
                    id,
                    path: entry.path.clone(),
                    language: None,
                    analysis: AnalysisStatus::None,
                });
            }
            InventoryOutcome::Included => {
                let language = language::detect_language_with_config(&path, config)
                    .unwrap_or("unknown")
                    .to_owned();
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        let detected = std::panic::catch_unwind(|| {
                            detect_components(&content, &language, &entry.path)
                        });
                        match detected {
                            Ok(mut detected) => {
                                components.append(&mut detected);
                                file_contents.insert(entry.path.clone(), content);
                                source_files.push(SourceFile {
                                    id,
                                    path: entry.path.clone(),
                                    language: Some(language.clone()),
                                    analysis: AnalysisStatus::Parsed {
                                        analyzer: format!("builtin-{language}"),
                                    },
                                });
                            }
                            Err(_) => {
                                let diagnostic_id = v2::stable_id(
                                    "diagnostic",
                                    &["failed", &entry.path, "analyzer panic"],
                                );
                                entry.outcome = InventoryOutcome::Failed {
                                    diagnostic_id: diagnostic_id.clone(),
                                };
                                diagnostics.push(Diagnostic::FailedFile {
                                    id: diagnostic_id,
                                    path: entry.path.clone(),
                                    message: "analyzer failed".into(),
                                });
                                source_files.push(SourceFile {
                                    id,
                                    path: entry.path.clone(),
                                    language: Some(language),
                                    analysis: AnalysisStatus::Failed {
                                        message: "analyzer failed".into(),
                                    },
                                });
                            }
                        }
                    }
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
                        });
                    }
                }
            }
        }
    }

    components.sort_by(|a, b| {
        (
            &a.source.file,
            a.source.line_start,
            &a.name,
            kind_name(&a.kind),
        )
            .cmp(&(
                &b.source.file,
                b.source.line_start,
                &b.name,
                kind_name(&b.kind),
            ))
    });
    assign_scan_component_ids(&mut components);
    let file_ids: HashMap<_, _> = source_files
        .iter()
        .map(|file| (file.path.clone(), file.id.clone()))
        .collect();
    let mut sibling_ordinals: HashMap<(String, String, String), usize> = HashMap::new();
    let mut old_to_new = HashMap::new();
    let entities: Vec<_> = components
        .iter()
        .filter_map(|component| {
            let file_id = file_ids.get(&component.source.file)?.clone();
            let declaration_kind = kind_name(&component.kind).to_owned();
            let key = (
                component.source.file.clone(),
                component.name.clone(),
                declaration_kind.clone(),
            );
            let ordinal = sibling_ordinals.entry(key).or_default();
            let id = v2::entity_id(&file_id, &component.name, &declaration_kind, *ordinal);
            *ordinal += 1;
            old_to_new.insert(component.id.clone(), id.clone());
            let line = component.source.line_start.unwrap_or(1);
            Some(CodeEntity {
                id,
                name: component.name.clone(),
                qualified_name: component.name.clone(),
                declaration_kind,
                file_id: file_id.clone(),
                scope_id: v2::scope_id(&file_id),
                owner_id: None,
                span: SourceSpan {
                    file_id,
                    start_line: line,
                    start_column: 1,
                    end_line: component.source.line_end.unwrap_or(line),
                    end_column: 1,
                },
                attributes: component_attributes(component),
            })
        })
        .collect();

    let legacy_edges = relationships::infer_edges(&components, &file_contents)
        .into_iter()
        .chain(relationships::infer_flow_edges(&components, &file_contents))
        .chain(relationships::infer_call_edges(&components, &file_contents));
    let mut v2_relationships = Vec::new();
    for edge in legacy_edges {
        let (Some(source), Some(target)) =
            (old_to_new.get(&edge.from_id), old_to_new.get(&edge.to_id))
        else {
            continue;
        };
        let origin = "inferred".to_string();
        let kind = relationship_kind(edge.label.as_deref());
        let id = v2::relationship_id(source, target, kind, &origin);
        v2_relationships.push(make_relationship(
            kind,
            id,
            source.clone(),
            target.clone(),
            origin,
        ));
    }
    v2_relationships.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    v2_relationships.dedup_by(|a, b| a.sort_key() == b.sort_key());

    let counts = inventory_counts(&inventory);
    Ok(Snapshot {
        manifest: Manifest {
            schema_version: "2".into(),
            repository,
            scanned_at: chrono::Utc::now().to_rfc3339(),
            root: root.display().to_string(),
            tool_version: env!("CARGO_PKG_VERSION").into(),
            analyzer_versions: Default::default(),
            inventory: counts,
            inventory_entries: inventory.entries,
        },
        source_files,
        entities,
        modules: Vec::new(),
        relationships: v2_relationships,
        unresolved_references: Vec::new(),
        evidence: Vec::new(),
        claims: Vec::new(),
        payload_contracts: Vec::new(),
        diagnostics,
        projections: Vec::new(),
        findings: Vec::new(),
    })
}

fn detect_components(content: &str, language: &str, file: &str) -> Vec<DetectedComponent> {
    let mut components = Vec::new();
    components.extend(models::detect_models(content, language, file));
    components.extend(services::detect_services(content, language, file));
    components.extend(transports::detect_transports(content, language, file));
    components.extend(transforms::detect_transforms(content, language, file));
    components.extend(prompts::detect_prompts(content, language, file));
    components
}

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

fn kind_name(kind: &ComponentKind) -> &'static str {
    match kind {
        ComponentKind::Model => "model",
        ComponentKind::Service => "service",
        ComponentKind::Transport => "transport",
        ComponentKind::Transform => "transform",
        ComponentKind::Prompt => "prompt",
    }
}

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

fn relationship_kind(label: Option<&str>) -> &'static str {
    match label.unwrap_or_default().to_ascii_lowercase().as_str() {
        "imports" | "import" => "imports",
        "calls" | "call" => "calls",
        "contains" => "contains",
        "depends_on" | "depends on" => "depends_on",
        "handles" => "handles",
        "persists" => "persists",
        "transforms" => "transforms",
        "consumes" => "consumes",
        "produces" => "produces",
        "dispatches" => "dispatches",
        "invokes_prompt" | "invokes prompt" => "invokes_prompt",
        _ => "references",
    }
}

fn make_relationship(
    kind: &str,
    id: v2::RelationshipId,
    source: v2::EntityId,
    target: v2::EntityId,
    origin: String,
) -> Relationship {
    macro_rules! rel {
        ($variant:ident) => {
            Relationship::$variant {
                id,
                source,
                target,
                origin,
                evidence_id: None,
            }
        };
    }
    match kind {
        "imports" => rel!(Imports),
        "calls" => rel!(Calls),
        "contains" => rel!(Contains),
        "depends_on" => rel!(DependsOn),
        "handles" => rel!(Handles),
        "persists" => rel!(Persists),
        "transforms" => rel!(Transforms),
        "consumes" => rel!(Consumes),
        "produces" => rel!(Produces),
        "dispatches" => rel!(Dispatches),
        "invokes_prompt" => rel!(InvokesPrompt),
        _ => rel!(References),
    }
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
