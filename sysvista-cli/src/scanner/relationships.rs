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

// Patterns for function call detection
static MODULE_CALL_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\w+)\.(\w+)\s*\(").unwrap());

static BACKGROUND_DISPATCH_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"background_tasks\.add_task\s*\(\s*(\w+)").unwrap());

static AWAIT_CALL_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"await\s+(\w+)\s*\(").unwrap());

// Python import: "from .foo import bar" or "from foo import bar"
static PYTHON_IMPORT_ALIAS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(?:from\s+\.?(\S+)\s+)?import\s+(\w+)(?:\s+as\s+(\w+))?").unwrap());

/// Build a map from module alias to imported module path for a single file
fn build_import_index(content: &str) -> HashMap<String, String> {
    let mut index = HashMap::new();
    for cap in PYTHON_IMPORT_ALIAS.captures_iter(content) {
        let module_path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let imported_name = &cap[2];
        let alias = cap.get(3).map(|m| m.as_str()).unwrap_or(imported_name);
        // Map alias to module path (e.g., "crud" -> "src.crud" or just "crud")
        if !module_path.is_empty() {
            index.insert(alias.to_string(), module_path.to_string());
        } else {
            index.insert(alias.to_string(), imported_name.to_string());
        }
    }
    index
}

/// Infer call edges: transport → service function calls and dispatch edges.
/// Scans handler bodies for module.function() calls, background task dispatches,
/// and awaited function calls.
pub fn infer_call_edges(
    components: &[DetectedComponent],
    file_contents: &HashMap<String, String>,
) -> Vec<DetectedEdge> {
    use crate::output::schema::ComponentKind;

    let mut edges = Vec::new();
    let name_index = build_name_index(components);

    // Build file→components index
    let mut by_file: HashMap<&str, Vec<&DetectedComponent>> = HashMap::new();
    for comp in components {
        by_file.entry(comp.source.file.as_str()).or_default().push(comp);
    }

    // Build a map from file stem (last path segment without extension) to file path
    // This helps resolve "from .crud import ..." → find components in crud.py
    let mut stem_to_file: HashMap<String, Vec<String>> = HashMap::new();
    for file in file_contents.keys() {
        if let Some(stem) = std::path::Path::new(file)
            .file_stem()
            .and_then(|s| s.to_str())
        {
            stem_to_file.entry(stem.to_string()).or_default().push(file.clone());
        }
        // Also index by last path segment for dotted imports like "app.crud"
        if let Some(last_dir) = std::path::Path::new(file)
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
        {
            stem_to_file.entry(last_dir.to_string()).or_default().push(file.clone());
        }
    }

    for (file, comps) in &by_file {
        let transports: Vec<&&DetectedComponent> = comps.iter()
            .filter(|c| c.kind == ComponentKind::Transport)
            .collect();

        if transports.is_empty() {
            continue;
        }

        let content = match file_contents.get(*file) {
            Some(c) => c,
            None => continue,
        };

        let import_index = build_import_index(content);
        let lines: Vec<&str> = content.lines().collect();

        for tp in &transports {
            let start_line = tp.source.line_start.unwrap_or(1) as usize;
            let end_line = (start_line + 80).min(lines.len());
            let start_idx = if start_line > 0 { start_line - 1 } else { 0 };
            let body = lines[start_idx..end_line].join("\n");

            // 1. Module function calls: module.function()
            for cap in MODULE_CALL_PATTERN.captures_iter(&body) {
                let module_alias = &cap[1];
                let func_name = &cap[2];

                // Skip common non-module calls
                if ["self", "cls", "db", "session", "response", "request", "app", "logger", "log"].contains(&module_alias) {
                    continue;
                }

                // Try to resolve module via import index
                let resolved = import_index.get(module_alias);

                // Find target component by function name
                let target = resolve_call_target(
                    func_name,
                    resolved.map(|s| s.as_str()),
                    module_alias,
                    &name_index,
                    components,
                    &stem_to_file,
                    &by_file,
                );

                if let Some(target_id) = target {
                    if target_id != tp.id {
                        edges.push(DetectedEdge {
                            from_id: tp.id.clone(),
                            to_id: target_id,
                            label: Some("calls".to_string()),
                            payload_type: None,
                        });
                    }
                }
            }

            // 2. Background dispatch: background_tasks.add_task(func, ...)
            for cap in BACKGROUND_DISPATCH_PATTERN.captures_iter(&body) {
                let func_name = &cap[1];
                if let Some(targets) = name_index.get(func_name) {
                    for &idx in targets {
                        if components[idx].id != tp.id {
                            edges.push(DetectedEdge {
                                from_id: tp.id.clone(),
                                to_id: components[idx].id.clone(),
                                label: Some("dispatches".to_string()),
                                payload_type: None,
                            });
                        }
                    }
                }
            }

            // 3. Awaited calls: await function()
            for cap in AWAIT_CALL_PATTERN.captures_iter(&body) {
                let func_name = &cap[1];

                // Skip common awaited non-component calls
                if ["fetch", "sleep", "gather", "wait", "commit", "execute", "flush", "refresh", "close"].contains(&func_name) {
                    continue;
                }

                if let Some(targets) = name_index.get(func_name) {
                    for &idx in targets {
                        if components[idx].id != tp.id {
                            edges.push(DetectedEdge {
                                from_id: tp.id.clone(),
                                to_id: components[idx].id.clone(),
                                label: Some("calls".to_string()),
                                payload_type: None,
                            });
                        }
                    }
                }
            }
        }
    }

    // Deduplicate
    edges.sort_by(|a, b| (&a.from_id, &a.to_id, &a.label).cmp(&(&b.from_id, &b.to_id, &b.label)));
    edges.dedup_by(|a, b| a.from_id == b.from_id && a.to_id == b.to_id && a.label == b.label);

    edges
}

/// Resolve a function call target to a component ID
fn resolve_call_target(
    func_name: &str,
    resolved_module: Option<&str>,
    module_alias: &str,
    name_index: &HashMap<String, Vec<usize>>,
    components: &[DetectedComponent],
    stem_to_file: &HashMap<String, Vec<String>>,
    by_file: &HashMap<&str, Vec<&DetectedComponent>>,
) -> Option<String> {
    // Strategy 1: If we have a resolved module path, find components in files matching that module
    let module_key = resolved_module.unwrap_or(module_alias);
    // Get the last segment of dotted path (e.g., "app.crud" -> "crud")
    let module_stem = module_key.rsplit('.').next().unwrap_or(module_key);

    if let Some(files) = stem_to_file.get(module_stem) {
        for file in files {
            if let Some(comps) = by_file.get(file.as_str()) {
                for comp in comps {
                    if comp.name == func_name {
                        return Some(comp.id.clone());
                    }
                }
            }
        }
    }

    // Strategy 2: Fall back to name-only matching
    if let Some(targets) = name_index.get(func_name) {
        if targets.len() == 1 {
            return Some(components[targets[0]].id.clone());
        }
    }

    None
}
