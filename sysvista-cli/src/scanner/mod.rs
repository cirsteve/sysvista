pub mod file_walker;
pub mod language;
pub mod models;
pub mod relationships;
pub mod services;
pub mod transforms;
pub mod transports;
pub mod workflows;

use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Instant;

use crate::output::schema::{DetectedComponent, ScanStats, SysVistaOutput};

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

        all_components.extend(components);
    }

    // Deduplicate components by ID (multiple patterns can match the same definition)
    let mut seen_ids = HashSet::new();
    all_components.retain(|c| seen_ids.insert(c.id.clone()));

    // Infer edges
    let mut edges = relationships::infer_edges(&all_components, &file_contents);

    // Infer flow edges (handles, persists, transforms, consumes, produces) and merge.
    // Skip flow edges where an import/reference edge already exists,
    // but always keep payload edges (consumes/produces) since they carry unique meaning.
    let flow_edges = relationships::infer_flow_edges(&all_components, &file_contents);
    let existing_pairs: HashSet<(String, String)> = edges
        .iter()
        .map(|e| (e.from_id.clone(), e.to_id.clone()))
        .collect();
    for fe in flow_edges {
        let is_payload = fe.label.as_deref() == Some("consumes")
            || fe.label.as_deref() == Some("produces");
        if is_payload || !existing_pairs.contains(&(fe.from_id.clone(), fe.to_id.clone())) {
            edges.push(fe);
        }
    }

    // Infer call/dispatch edges and merge.
    // Always allow calls/dispatches edges through (like payload edges).
    let call_edges = relationships::infer_call_edges(&all_components, &file_contents);
    let existing_pairs: HashSet<(String, String)> = edges
        .iter()
        .map(|e| (e.from_id.clone(), e.to_id.clone()))
        .collect();
    for ce in call_edges {
        let is_call = ce.label.as_deref() == Some("calls")
            || ce.label.as_deref() == Some("dispatches");
        if is_call || !existing_pairs.contains(&(ce.from_id.clone(), ce.to_id.clone())) {
            edges.push(ce);
        }
    }

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
