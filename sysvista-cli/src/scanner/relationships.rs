use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::output::schema::{DetectedComponent, DetectedEdge};

// Import patterns for various languages
static IMPORT_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // TypeScript/JavaScript: import ... from "..."
        Regex::new(r#"(?m)import\s+.*?from\s+['"]([^'"]+)['"]"#).unwrap(),
        // TypeScript/JavaScript: require("...")
        Regex::new(r#"(?m)require\s*\(\s*['"]([^'"]+)['"]"#).unwrap(),
        // Rust: use crate::...
        Regex::new(r"(?m)^use\s+(?:crate::)?(\S+);").unwrap(),
        // Python: from ... import ...
        Regex::new(r"(?m)^from\s+(\S+)\s+import").unwrap(),
        // Go: import "..."
        Regex::new(r#"(?m)import\s+(?:\w+\s+)?"([^"]+)""#).unwrap(),
    ]
});

/// Build a map from filename stem to components in that file
fn build_file_index(components: &[DetectedComponent]) -> HashMap<String, Vec<usize>> {
    let mut index: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, comp) in components.iter().enumerate() {
        let file = &comp.source.file;
        // Index by full relative path
        index.entry(file.clone()).or_default().push(i);
        // Index by file stem (e.g. "user.service" from "src/services/user.service.ts")
        if let Some(stem) = std::path::Path::new(file)
            .file_stem()
            .and_then(|s| s.to_str())
        {
            index.entry(stem.to_string()).or_default().push(i);
        }
    }
    index
}

/// Build a map from component name to component index
fn build_name_index(components: &[DetectedComponent]) -> HashMap<String, Vec<usize>> {
    let mut index: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, comp) in components.iter().enumerate() {
        index.entry(comp.name.clone()).or_default().push(i);
    }
    index
}

/// Read file contents and extract import paths, returning resolved file paths
fn extract_imports(content: &str) -> Vec<String> {
    let mut imports = Vec::new();
    for pattern in IMPORT_PATTERNS.iter() {
        for cap in pattern.captures_iter(content) {
            imports.push(cap[1].to_string());
        }
    }
    imports
}

/// Infer edges between components based on imports and type references
pub fn infer_edges(
    components: &[DetectedComponent],
    file_contents: &HashMap<String, String>,
) -> Vec<DetectedEdge> {
    let mut edges = Vec::new();
    let file_index = build_file_index(components);
    let name_index = build_name_index(components);

    // For each file, find imports and create edges
    for (file, content) in file_contents {
        let imports = extract_imports(content);
        let source_components: Vec<usize> = file_index.get(file.as_str()).cloned().unwrap_or_default();

        for import_path in &imports {
            // Try to resolve the import to a file in our index
            let import_stem = std::path::Path::new(import_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(import_path);

            // Find target components that might match this import
            let target_indices: Vec<usize> = file_index
                .get(import_stem)
                .cloned()
                .or_else(|| {
                    // Try matching by last segment of path
                    let last_segment = import_path.rsplit('/').next().unwrap_or(import_path);
                    file_index.get(last_segment).cloned()
                })
                .unwrap_or_default();

            for &src_idx in &source_components {
                for &tgt_idx in &target_indices {
                    if src_idx != tgt_idx {
                        edges.push(DetectedEdge {
                            from_id: components[src_idx].id.clone(),
                            to_id: components[tgt_idx].id.clone(),
                            label: Some("imports".to_string()),
                            payload_type: None,
                        });
                    }
                }
            }
        }

        // Look for type name references in file content
        for &src_idx in &source_components {
            for (name, target_indices) in &name_index {
                // Skip self-references and very short names (likely false positives)
                if name.len() < 3 {
                    continue;
                }
                // Check if this type name appears in the file content as a word boundary match
                let pattern = format!(r"\b{}\b", regex::escape(name));
                if let Ok(re) = Regex::new(&pattern) {
                    let matches: Vec<_> = re.find_iter(content).collect();
                    // Need at least 2 matches to infer a reference (one is likely the definition)
                    let is_definition_file = target_indices
                        .iter()
                        .any(|&ti| components[ti].source.file == *file);
                    let threshold = if is_definition_file { 2 } else { 1 };

                    if matches.len() >= threshold {
                        for &tgt_idx in target_indices {
                            if src_idx != tgt_idx
                                && components[tgt_idx].source.file != *file
                            {
                                edges.push(DetectedEdge {
                                    from_id: components[src_idx].id.clone(),
                                    to_id: components[tgt_idx].id.clone(),
                                    label: Some("references".to_string()),
                                    payload_type: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Deduplicate edges
    edges.sort_by(|a, b| (&a.from_id, &a.to_id).cmp(&(&b.from_id, &b.to_id)));
    edges.dedup_by(|a, b| a.from_id == b.from_id && a.to_id == b.to_id);

    edges
}

/// Infer flow edges that represent request-handling relationships:
/// - handles: service → transport (route defined in same file as service)
/// - persists: transport → model (handler body references model types)
/// - transforms: transform → model (transform references model types)
pub fn infer_flow_edges(
    components: &[DetectedComponent],
    file_contents: &HashMap<String, String>,
) -> Vec<DetectedEdge> {
    use crate::output::schema::ComponentKind;

    let mut edges = Vec::new();

    // Group components by source file
    let mut by_file: HashMap<&str, Vec<&DetectedComponent>> = HashMap::new();
    for comp in components {
        by_file.entry(comp.source.file.as_str()).or_default().push(comp);
    }

    // Collect all model names for body scanning
    let model_names: Vec<(&str, &str)> = components
        .iter()
        .filter(|c| c.kind == ComponentKind::Model && c.name.len() >= 3)
        .map(|c| (c.id.as_str(), c.name.as_str()))
        .collect();

    for (file, comps) in &by_file {
        let services: Vec<&&DetectedComponent> = comps.iter().filter(|c| c.kind == ComponentKind::Service).collect();
        let transports: Vec<&&DetectedComponent> = comps.iter().filter(|c| c.kind == ComponentKind::Transport).collect();
        let transforms: Vec<&&DetectedComponent> = comps.iter().filter(|c| c.kind == ComponentKind::Transform).collect();

        // service --handles--> transport (same file)
        for svc in &services {
            for tp in &transports {
                edges.push(DetectedEdge {
                    from_id: svc.id.clone(),
                    to_id: tp.id.clone(),
                    label: Some("handles".to_string()),
                    payload_type: None,
                });
            }
        }

        // transport --persists--> model (handler body references model name)
        if let Some(content) = file_contents.get(*file) {
            let lines: Vec<&str> = content.lines().collect();

            for tp in &transports {
                let start_line = tp.source.line_start.unwrap_or(1) as usize;
                // Scan ~50 lines from the transport definition (handler body)
                let end_line = (start_line + 50).min(lines.len());
                let start_idx = if start_line > 0 { start_line - 1 } else { 0 };
                let body = lines[start_idx..end_line].join("\n");

                for &(model_id, model_name) in &model_names {
                    if model_id == tp.id {
                        continue;
                    }
                    let pattern = format!(r"\b{}\b", regex::escape(model_name));
                    if let Ok(re) = Regex::new(&pattern) {
                        if re.is_match(&body) {
                            edges.push(DetectedEdge {
                                from_id: tp.id.clone(),
                                to_id: model_id.to_string(),
                                label: Some("persists".to_string()),
                                payload_type: None,
                            });
                        }
                    }
                }
            }

            // transform --transforms--> model (transform body references model name)
            for tf in &transforms {
                let start_line = tf.source.line_start.unwrap_or(1) as usize;
                let end_line = (start_line + 50).min(lines.len());
                let start_idx = if start_line > 0 { start_line - 1 } else { 0 };
                let body = lines[start_idx..end_line].join("\n");

                for &(model_id, model_name) in &model_names {
                    if model_id == tf.id {
                        continue;
                    }
                    let pattern = format!(r"\b{}\b", regex::escape(model_name));
                    if let Ok(re) = Regex::new(&pattern) {
                        if re.is_match(&body) {
                            edges.push(DetectedEdge {
                                from_id: tf.id.clone(),
                                to_id: model_id.to_string(),
                                label: Some("transforms".to_string()),
                                payload_type: None,
                            });
                        }
                    }
                }
            }
        }
    }

    // Payload flow edges: match consumes/produces types to detected model names
    // model_names is Vec<(id, name)>, we need name→id
    let model_name_to_id: HashMap<&str, &str> = model_names
        .iter()
        .map(|&(id, name)| (name, id))
        .collect();

    for comp in components {
        if comp.kind != ComponentKind::Transport {
            continue;
        }

        // consumes: Model --consumes--> Transport (data flows into the transport)
        if let Some(ref consumes) = comp.consumes {
            for type_name in consumes {
                if let Some(&model_id) = model_name_to_id.get(type_name.as_str()) {
                    edges.push(DetectedEdge {
                        from_id: model_id.to_string(),
                        to_id: comp.id.clone(),
                        label: Some("consumes".to_string()),
                        payload_type: Some(type_name.clone()),
                    });
                }
            }
        }

        // produces: Transport --produces--> Model (data flows out)
        if let Some(ref produces) = comp.produces {
            for type_name in produces {
                if let Some(&model_id) = model_name_to_id.get(type_name.as_str()) {
                    edges.push(DetectedEdge {
                        from_id: comp.id.clone(),
                        to_id: model_id.to_string(),
                        label: Some("produces".to_string()),
                        payload_type: Some(type_name.clone()),
                    });
                }
            }
        }
    }

    // Deduplicate flow edges
    edges.sort_by(|a, b| (&a.from_id, &a.to_id, &a.label).cmp(&(&b.from_id, &b.to_id, &b.label)));
    edges.dedup_by(|a, b| a.from_id == b.from_id && a.to_id == b.to_id && a.label == b.label);

    edges
}
