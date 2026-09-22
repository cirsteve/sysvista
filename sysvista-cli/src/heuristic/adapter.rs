use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::Path,
};

use crate::{
    discovery::{Config, Inventory, InventoryOutcome},
    output::{
        schema::{ComponentKind, DetectedComponent},
        v2::{self, Claim, CodeEntity, Diagnostic, EntityId, Evidence, Relationship, SourceSpan},
    },
    scanner::{self, relationships, workflows},
};

#[derive(Debug, Default)]
pub struct HeuristicAnalysis {
    pub entities: Vec<CodeEntity>,
    pub relationships: Vec<Relationship>,
    pub evidence: Vec<Evidence>,
    pub claims: Vec<Claim>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Run the legacy regex detectors as one provenance-labelled analysis stage.
/// Detector implementations deliberately remain in `scanner`; this is only an adapter.
pub fn analyze(
    root: &Path,
    repository: &str,
    inventory: &Inventory,
    config: &Config,
) -> HeuristicAnalysis {
    let mut components = Vec::new();
    let mut contents = HashMap::new();
    let mut diagnostics = Vec::new();

    for entry in inventory
        .entries
        .iter()
        .filter(|entry| entry.outcome == InventoryOutcome::Included)
    {
        let path = Inventory::absolute_path(root, entry);
        let Some(language) = scanner::language::detect_language_with_config(&path, config) else {
            continue;
        };
        match fs::read_to_string(&path) {
            Ok(content) => {
                let detected = std::panic::catch_unwind(|| detect(&content, language, &entry.path));
                match detected {
                    Ok(mut found) => {
                        components.append(&mut found);
                        contents.insert(entry.path.clone(), content);
                    }
                    Err(_) => diagnostics.push(Diagnostic::FailedFile {
                        id: v2::stable_id("diagnostic", &["heuristic", &entry.path]),
                        path: entry.path.clone(),
                        message: "heuristic detector failed".into(),
                    }),
                }
            }
            Err(error) => diagnostics.push(Diagnostic::UnreadableFile {
                id: v2::stable_id(
                    "diagnostic",
                    &["heuristic", &entry.path, &error.to_string()],
                ),
                path: entry.path.clone(),
                message: error.to_string(),
            }),
        }
    }

    components.sort_by(|a, b| component_key(a).cmp(&component_key(b)));
    let mut seen = HashSet::new();
    components.retain(|component| {
        seen.insert((
            component.source.file.clone(),
            component.name.clone(),
            kind_name(&component.kind),
        ))
    });
    for (index, component) in components.iter_mut().enumerate() {
        component.id = v2::stable_id(
            "heuristic_component",
            &[
                &component.source.file,
                kind_name(&component.kind),
                &component.name,
                &index.to_string(),
            ],
        );
    }

    let mut old_to_new = HashMap::new();
    let mut component_files = HashMap::new();
    let mut ordinals = HashMap::new();
    let mut entities = Vec::new();
    let mut module_ids = HashMap::new();
    for (file, content) in &contents {
        let file_id = v2::file_id(repository, file);
        let id = v2::entity_id(&file_id, "<module>", "module", 0);
        module_ids.insert(file.clone(), id.clone());
        entities.push(CodeEntity {
            id,
            name: "<module>".into(),
            qualified_name: "<module>".into(),
            declaration_kind: "module".into(),
            file_id: file_id.clone(),
            scope_id: v2::scope_id(&file_id),
            owner_id: None,
            span: SourceSpan {
                file_id,
                start_line: 1,
                start_column: 1,
                end_line: content.lines().count().max(1) as u32,
                end_column: 1,
            },
            attributes: Default::default(),
        });
    }
    for component in &components {
        let file_id = v2::file_id(repository, &component.source.file);
        let declaration_kind = kind_name(&component.kind);
        let ordinal = ordinals
            .entry((
                component.source.file.clone(),
                component.name.clone(),
                declaration_kind,
            ))
            .or_insert(0usize);
        let id = v2::entity_id(&file_id, &component.name, declaration_kind, *ordinal);
        *ordinal += 1;
        old_to_new.insert(component.id.clone(), id.clone());
        component_files.insert(component.id.clone(), component.source.file.clone());
        let line = component.source.line_start.unwrap_or(1);
        entities.push(CodeEntity {
            id,
            name: component.name.clone(),
            qualified_name: component.name.clone(),
            declaration_kind: declaration_kind.into(),
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
            attributes: attributes(component),
        });
    }

    let edges: Vec<_> = relationships::infer_edges(&components, &contents)
        .into_iter()
        .chain(relationships::infer_flow_edges(&components, &contents))
        .chain(relationships::infer_call_edges(&components, &contents))
        .collect();
    let legacy_workflows = workflows::infer_workflows(&components, &edges);
    let mut output_relationships = Vec::new();
    let mut evidence = Vec::new();
    let mut claims = Vec::new();
    for edge in &edges {
        let kind = relationship_kind(edge.label.as_deref());
        let source = if kind == "imports" {
            component_files
                .get(&edge.from_id)
                .and_then(|file| module_ids.get(file))
        } else {
            old_to_new.get(&edge.from_id)
        };
        let (Some(source), Some(target)) = (source, old_to_new.get(&edge.to_id)) else {
            continue;
        };
        let rule = rule_name(kind);
        let evidence_id = v2::stable_id(
            "evidence",
            &["heuristic", source.as_ref(), target.as_ref(), kind, rule],
        );
        evidence.push(Evidence::Analyzer {
            id: evidence_id.clone(),
            analyzer: "builtin-heuristic".into(),
            detail: format!("legacy detector rule {rule}"),
            origin: Some("heuristic".into()),
            confidence: Some(
                if rule == "model_name_match" {
                    "low"
                } else {
                    "medium"
                }
                .into(),
            ),
            rule: Some(rule.into()),
        });
        let id = v2::relationship_id(source, target, kind, "heuristic");
        output_relationships.push(make_relationship(
            kind,
            id,
            source.clone(),
            target.clone(),
            Some(evidence_id.clone()),
        ));
        claims.push(Claim {
            id: v2::stable_id("claim", &[source.as_ref(), target.as_ref(), kind, rule]),
            subject: source.clone(),
            predicate: rule.into(),
            object: serde_json::json!({"relationship_kind": kind, "target": target}),
            evidence_ids: vec![evidence_id],
        });
    }

    for workflow in legacy_workflows {
        let entity_ids: BTreeSet<_> = workflow
            .steps
            .iter()
            .filter_map(|step| old_to_new.get(&step.component_id))
            .cloned()
            .collect();
        let entity_set: HashSet<_> = workflow
            .steps
            .iter()
            .map(|step| step.component_id.as_str())
            .collect();
        let edge_ids: BTreeSet<_> = edges
            .iter()
            .filter(|edge| {
                entity_set.contains(edge.from_id.as_str())
                    && entity_set.contains(edge.to_id.as_str())
            })
            .filter_map(|edge| {
                let source = old_to_new.get(&edge.from_id)?;
                let target = old_to_new.get(&edge.to_id)?;
                Some(v2::relationship_id(
                    source,
                    target,
                    relationship_kind(edge.label.as_deref()),
                    "heuristic",
                ))
            })
            .collect();
        let Some(subject) = old_to_new.get(&workflow.entry_point_id).cloned() else {
            continue;
        };
        claims.push(Claim { id: v2::stable_id("claim", &["heuristic_traversal", &workflow.id]), subject, predicate: "HeuristicTraversal".into(), object: serde_json::json!({"name": workflow.name, "entity_ids": entity_ids, "relationship_ids": edge_ids}), evidence_ids: Vec::new() });
    }
    output_relationships.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    output_relationships.dedup_by(|a, b| a.sort_key() == b.sort_key());
    evidence.sort_by(|a, b| evidence_id(a).cmp(evidence_id(b)));
    evidence.dedup_by_key(|item| evidence_id(item).to_owned());
    claims.sort_by(|a, b| a.id.cmp(&b.id));

    HeuristicAnalysis {
        entities,
        relationships: output_relationships,
        evidence,
        claims,
        diagnostics,
    }
}

fn detect(content: &str, language: &str, file: &str) -> Vec<DetectedComponent> {
    let mut result = Vec::new();
    result.extend(scanner::models::detect_models(content, language, file));
    result.extend(scanner::services::detect_services(content, language, file));
    result.extend(scanner::transports::detect_transports(
        content, language, file,
    ));
    result.extend(scanner::transforms::detect_transforms(
        content, language, file,
    ));
    result.extend(scanner::prompts::detect_prompts(content, language, file));
    result
}

fn component_key(component: &DetectedComponent) -> (&str, Option<u32>, &str, &'static str) {
    (
        &component.source.file,
        component.source.line_start,
        &component.name,
        kind_name(&component.kind),
    )
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
fn relationship_kind(label: Option<&str>) -> &'static str {
    match label.unwrap_or_default() {
        "imports" => "imports",
        "calls" => "calls",
        "handles" => "handles",
        "persists" => "persists",
        "transforms" => "transforms",
        "consumes" => "consumes",
        "produces" => "produces",
        "dispatches" => "dispatches",
        "invokes_prompt" => "invokes_prompt",
        _ => "references",
    }
}
fn rule_name(kind: &str) -> &'static str {
    match kind {
        "persists" => "model_name_match",
        "imports" => "import_path_match",
        "calls" => "call_name_match",
        "handles" => "co_location_match",
        "transforms" => "transform_model_match",
        "consumes" => "payload_input_match",
        "produces" => "payload_output_match",
        "dispatches" => "dispatch_name_match",
        "invokes_prompt" => "prompt_name_match",
        _ => "name_reference_match",
    }
}
fn attributes(
    component: &DetectedComponent,
) -> std::collections::BTreeMap<String, serde_json::Value> {
    let mut result = std::collections::BTreeMap::new();
    for (key, value) in &component.metadata {
        result.insert(key.clone(), value.clone().into());
    }
    result
}
fn evidence_id(evidence: &Evidence) -> &str {
    match evidence {
        Evidence::Source { id, .. } | Evidence::Text { id, .. } | Evidence::Analyzer { id, .. } => {
            id
        }
    }
}

fn make_relationship(
    kind: &str,
    id: v2::RelationshipId,
    source: EntityId,
    target: EntityId,
    evidence_id: Option<String>,
) -> Relationship {
    macro_rules! rel {
        ($variant:ident) => {
            Relationship::$variant {
                id,
                source,
                target,
                origin: "heuristic".into(),
                evidence_id,
            }
        };
    }
    match kind {
        "imports" => rel!(Imports),
        "calls" => rel!(Calls),
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
