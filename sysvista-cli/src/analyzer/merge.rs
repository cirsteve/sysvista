use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::contract::{AnalyzeResponse, AnalyzerSpan};
use crate::{
    heuristic::HeuristicAnalysis,
    output::v2::{
        self, Claim, CodeEntity, Diagnostic, EntityId, Evidence, PayloadContract, Relationship,
        SourceSpan, UnresolvedReference,
    },
};

#[derive(Debug, Default)]
pub struct MergedAnalysis {
    pub entities: Vec<CodeEntity>,
    pub relationships: Vec<Relationship>,
    pub unresolved: Vec<UnresolvedReference>,
    pub evidence: Vec<Evidence>,
    pub claims: Vec<Claim>,
    pub payloads: Vec<PayloadContract>,
    pub diagnostics: Vec<Diagnostic>,
}

/// What the merge needs to know about the scan that produced the analyzer response.
pub struct MergeContext<'a> {
    pub repository: &'a str,
    /// Absolute scan root, scrubbed from analyzer messages.
    pub root: &'a str,
    /// Normalized paths of every inventoried file; analyzer spans must name one of them.
    pub files: &'a BTreeSet<String>,
}

/// Number of unmatched analyzer keys named individually before they are summarized.
const UNMATCHED_KEY_LIMIT: usize = 20;

pub fn merge(
    context: &MergeContext,
    response: AnalyzeResponse,
    heuristic: HeuristicAnalysis,
) -> MergedAnalysis {
    let repository = context.repository;
    let HeuristicAnalysis {
        entities: heuristic_entities,
        relationships: heuristic_relationships,
        evidence: heuristic_evidence,
        claims: heuristic_claims,
        diagnostics: heuristic_diagnostics,
    } = heuristic;
    let mut issues = MergeIssues::default();
    let response_entities: Vec<_> = response
        .entities
        .iter()
        .filter(|entity| issues.keep_file(context, &entity.file))
        .collect();
    let mut key_to_id = HashMap::new();
    for entity in &response_entities {
        let file_id = v2::file_id(repository, &entity.file);
        key_to_id.insert(
            key(entity),
            v2::entity_id(
                &file_id,
                &entity.ownership_chain,
                &entity.declaration_kind,
                entity.discriminator,
            ),
        );
    }
    let mut analyzer_entities = Vec::new();
    for entity in &response_entities {
        let file_id = v2::file_id(repository, &entity.file);
        let id = key_to_id[&key(entity)].clone();
        let owner_id = entity
            .owner_key
            .as_ref()
            .and_then(|owner| issues.lookup(&key_to_id, owner));
        analyzer_entities.push(CodeEntity {
            id,
            name: entity.name.clone(),
            qualified_name: entity.ownership_chain.clone(),
            declaration_kind: entity.declaration_kind.clone(),
            file_id: file_id.clone(),
            scope_id: v2::scope_id(&file_id),
            owner_id,
            span: SourceSpan {
                file_id,
                start_line: entity.start_line,
                start_column: entity.start_column,
                end_line: entity.end_line,
                end_column: entity.end_column,
            },
            attributes: entity
                .attributes
                .clone()
                .into_iter()
                .collect::<BTreeMap<_, _>>(),
        });
    }
    let identity_map: HashMap<_, _> = heuristic_entities
        .iter()
        .filter_map(|heuristic| {
            analyzer_entities
                .iter()
                .filter(|resolved| {
                    resolved.file_id == heuristic.file_id
                        && resolved.name == heuristic.name
                        && (resolved.declaration_kind == "module"
                            || spans_overlap(&resolved.span, &heuristic.span))
                })
                .min_by_key(|resolved| resolved.span.start_line.abs_diff(heuristic.span.start_line))
                .map(|resolved| (heuristic.id.clone(), resolved.id.clone()))
        })
        .collect();
    let mut entities: Vec<_> = heuristic_entities
        .into_iter()
        .filter(|entity| !identity_map.contains_key(&entity.id))
        .collect();
    entities.extend(analyzer_entities);
    entities.sort_by(|a, b| a.id.cmp(&b.id));
    entities.dedup_by(|a, b| a.id == b.id);
    let mut relationship_identity = HashMap::new();
    let mut relationships: Vec<_> = heuristic_relationships
        .into_iter()
        .map(|relationship| {
            let old_id = relationship_id_ref(&relationship).clone();
            let remapped = remap_relationship(relationship, &identity_map);
            relationship_identity.insert(old_id, relationship_id_ref(&remapped).clone());
            remapped
        })
        .collect();
    // One relationship per (origin, source, target, kind); every call site or binding
    // behind it becomes a site on that relationship's single evidence record.
    let mut analyzer_evidence: BTreeMap<String, AnalyzerEvidence> = BTreeMap::new();
    for edge in &response.relationships {
        let Some(target_key) = edge.target.as_ref() else {
            continue;
        };
        let (Some(source), Some(target)) = (
            issues.lookup(&key_to_id, &edge.source),
            issues.lookup(&key_to_id, target_key),
        ) else {
            continue;
        };
        let evidence_id = v2::stable_id(
            "evidence",
            &[&edge.origin, source.as_ref(), target.as_ref(), &edge.kind],
        );
        let record = analyzer_evidence.entry(evidence_id.clone()).or_insert_with(|| {
            relationships.push(make_relationship(
                &edge.kind,
                source.clone(),
                target.clone(),
                edge.origin.clone(),
                Some(evidence_id),
            ));
            AnalyzerEvidence {
                kind: edge.kind.clone(),
                origin: edge.origin.clone(),
                ..Default::default()
            }
        });
        record.names.extend(edge.name.clone());
        if let Some(span) = edge.span.clone() {
            if issues.keep_file(context, &span.file) {
                record.sites.push(convert_span(repository, span));
            }
        }
    }
    let mut evidence = heuristic_evidence;
    for (id, record) in analyzer_evidence {
        evidence.push(record.into_evidence(id, &response.analyzer_version));
    }
    relationships
        .sort_by(|a, b| (rank(origin(a)), a.sort_key()).cmp(&(rank(origin(b)), b.sort_key())));
    relationships.dedup_by(|a, b| a.sort_key() == b.sort_key());
    let unresolved = response
        .unresolved
        .into_iter()
        .filter_map(|item| {
            let source = issues.lookup(&key_to_id, &item.source)?;
            issues.keep_file(context, &item.span.file).then(|| UnresolvedReference {
                name: item.name,
                source,
                span: convert_span(repository, item.span),
                reason: item.reason,
            })
        })
        .collect();
    let payloads = response
        .payloads
        .into_iter()
        .map(|payload| PayloadContract {
            name: payload.name,
            schema: None,
            producer_ids: payload
                .producers
                .iter()
                .filter_map(|key| issues.lookup(&key_to_id, key))
                .collect(),
            consumer_ids: payload
                .consumers
                .iter()
                .filter_map(|key| issues.lookup(&key_to_id, key))
                .collect(),
        })
        .collect();
    let mut diagnostics = heuristic_diagnostics;
    for diagnostic in response.diagnostics {
        let span = diagnostic
            .span
            .filter(|span| issues.keep_file(context, &span.file))
            .map(|span| convert_span(repository, span));
        diagnostics.push(Diagnostic::AnalyzerIssue {
            message: scrub(&diagnostic.message, context.root),
            severity: diagnostic.severity,
            span,
        });
    }
    diagnostics.extend(issues.into_diagnostics(context));
    MergedAnalysis {
        entities,
        relationships,
        unresolved,
        evidence,
        claims: heuristic_claims
            .into_iter()
            .map(|mut claim| {
                claim.subject = remap_id(claim.subject, &identity_map);
                remap_json(&mut claim.object, &identity_map, &relationship_identity);
                claim
            })
            .collect(),
        payloads,
        diagnostics,
    }
}

#[derive(Default)]
struct AnalyzerEvidence {
    kind: String,
    origin: String,
    names: BTreeSet<String>,
    sites: Vec<SourceSpan>,
}

impl AnalyzerEvidence {
    fn into_evidence(mut self, id: String, analyzer: &str) -> Evidence {
        self.sites.sort_by(|a, b| span_order(a).cmp(&span_order(b)));
        self.sites.dedup();
        Evidence::Analyzer {
            id,
            analyzer: analyzer.to_owned(),
            detail: if self.names.is_empty() {
                self.kind
            } else {
                self.names.into_iter().collect::<Vec<_>>().join(", ")
            },
            confidence: Some(if self.origin == "resolved" { "high" } else { "low" }.into()),
            origin: Some(self.origin),
            rule: None,
            sites: self.sites,
        }
    }
}

fn span_order(span: &SourceSpan) -> (&v2::FileId, u32, u32, u32, u32) {
    (&span.file_id, span.start_line, span.start_column, span.end_line, span.end_column)
}

/// Analyzer output the merge could not use, reported instead of silently dropped.
#[derive(Default)]
struct MergeIssues {
    unmatched_keys: BTreeSet<String>,
    outside_files: BTreeSet<String>,
}

impl MergeIssues {
    fn lookup(&mut self, key_to_id: &HashMap<String, EntityId>, key: &str) -> Option<EntityId> {
        let found = key_to_id.get(key).cloned();
        if found.is_none() {
            self.unmatched_keys.insert(key.to_owned());
        }
        found
    }

    fn keep_file(&mut self, context: &MergeContext, file: &str) -> bool {
        let known = context.files.contains(file);
        if !known {
            self.outside_files.insert(file.to_owned());
        }
        known
    }

    fn into_diagnostics(self, context: &MergeContext) -> Vec<Diagnostic> {
        let mut out = Vec::new();
        for key in self.unmatched_keys.iter().take(UNMATCHED_KEY_LIMIT) {
            // A key's file outside the inventory may be a host path; name only the rest.
            let key = match key.split_once('#') {
                Some((file, rest)) if !context.files.contains(file) => format!("<outside scan>#{rest}"),
                _ => key.clone(),
            };
            out.push(Diagnostic::Warning {
                message: format!("analyzer key {key} matches no emitted entity; records using it were dropped"),
                span: None,
            });
        }
        if self.unmatched_keys.len() > UNMATCHED_KEY_LIMIT {
            out.push(Diagnostic::Warning {
                message: format!(
                    "{} further analyzer keys matched no emitted entity",
                    self.unmatched_keys.len() - UNMATCHED_KEY_LIMIT
                ),
                span: None,
            });
        }
        if !self.outside_files.is_empty() {
            // Only the count: these paths may be absolute host paths.
            out.push(Diagnostic::Warning {
                message: format!(
                    "analyzer output named {} files outside the scan inventory; those records were dropped",
                    self.outside_files.len()
                ),
                span: None,
            });
        }
        out
    }
}

/// Remove the absolute scan root from a message so output does not depend on the checkout location.
fn scrub(message: &str, root: &str) -> String {
    let root = root.trim_end_matches('/');
    if root.is_empty() {
        return message.to_owned();
    }
    message.replace(&format!("{root}/"), "").replace(root, ".")
}

fn key(entity: &super::contract::AnalyzerEntity) -> String {
    format!(
        "{}#{}#{}#{}",
        entity.file, entity.ownership_chain, entity.declaration_kind, entity.discriminator
    )
}
fn spans_overlap(left: &SourceSpan, right: &SourceSpan) -> bool {
    left.file_id == right.file_id
        && left.start_line <= right.end_line
        && right.start_line <= left.end_line
}

fn remap_id(id: EntityId, identities: &HashMap<EntityId, EntityId>) -> EntityId {
    identities.get(&id).cloned().unwrap_or(id)
}

fn relationship_id_ref(relationship: &Relationship) -> &v2::RelationshipId {
    match relationship {
        Relationship::Imports { id, .. }
        | Relationship::References { id, .. }
        | Relationship::Calls { id, .. }
        | Relationship::Contains { id, .. }
        | Relationship::DependsOn { id, .. }
        | Relationship::Handles { id, .. }
        | Relationship::Persists { id, .. }
        | Relationship::Transforms { id, .. }
        | Relationship::Consumes { id, .. }
        | Relationship::Produces { id, .. }
        | Relationship::Dispatches { id, .. }
        | Relationship::InvokesPrompt { id, .. } => id,
    }
}

fn remap_relationship(
    relationship: Relationship,
    identities: &HashMap<EntityId, EntityId>,
) -> Relationship {
    macro_rules! remap {
        ($kind:literal, $source:ident, $target:ident, $origin:ident, $evidence_id:ident) => {
            make_relationship(
                $kind,
                remap_id($source, identities),
                remap_id($target, identities),
                $origin,
                $evidence_id,
            )
        };
    }
    match relationship {
        Relationship::Imports {
            source,
            target,
            origin,
            evidence_id,
            ..
        } => remap!("imports", source, target, origin, evidence_id),
        Relationship::References {
            source,
            target,
            origin,
            evidence_id,
            ..
        } => remap!("references", source, target, origin, evidence_id),
        Relationship::Calls {
            source,
            target,
            origin,
            evidence_id,
            ..
        } => remap!("calls", source, target, origin, evidence_id),
        Relationship::Contains {
            source,
            target,
            origin,
            evidence_id,
            ..
        } => remap!("contains", source, target, origin, evidence_id),
        Relationship::DependsOn {
            source,
            target,
            origin,
            evidence_id,
            ..
        } => remap!("depends_on", source, target, origin, evidence_id),
        Relationship::Handles {
            source,
            target,
            origin,
            evidence_id,
            ..
        } => remap!("handles", source, target, origin, evidence_id),
        Relationship::Persists {
            source,
            target,
            origin,
            evidence_id,
            ..
        } => remap!("persists", source, target, origin, evidence_id),
        Relationship::Transforms {
            source,
            target,
            origin,
            evidence_id,
            ..
        } => remap!("transforms", source, target, origin, evidence_id),
        Relationship::Consumes {
            source,
            target,
            origin,
            evidence_id,
            ..
        } => remap!("consumes", source, target, origin, evidence_id),
        Relationship::Produces {
            source,
            target,
            origin,
            evidence_id,
            ..
        } => remap!("produces", source, target, origin, evidence_id),
        Relationship::Dispatches {
            source,
            target,
            origin,
            evidence_id,
            ..
        } => remap!("dispatches", source, target, origin, evidence_id),
        Relationship::InvokesPrompt {
            source,
            target,
            origin,
            evidence_id,
            ..
        } => remap!("invokes_prompt", source, target, origin, evidence_id),
    }
}

fn remap_json(
    value: &mut serde_json::Value,
    entity_ids: &HashMap<EntityId, EntityId>,
    relationship_ids: &HashMap<v2::RelationshipId, v2::RelationshipId>,
) {
    match value {
        serde_json::Value::String(text) => {
            if let Some((_, replacement)) = entity_ids
                .iter()
                .find(|(id, _)| id.as_ref() == text.as_str())
            {
                *text = replacement.as_ref().to_owned();
            } else if let Some((_, replacement)) = relationship_ids
                .iter()
                .find(|(id, _)| id.as_ref() == text.as_str())
            {
                *text = replacement.as_ref().to_owned();
            }
        }
        serde_json::Value::Array(values) => values
            .iter_mut()
            .for_each(|value| remap_json(value, entity_ids, relationship_ids)),
        serde_json::Value::Object(values) => values
            .values_mut()
            .for_each(|value| remap_json(value, entity_ids, relationship_ids)),
        _ => {}
    }
}
fn convert_span(repository: &str, span: AnalyzerSpan) -> SourceSpan {
    SourceSpan {
        file_id: v2::file_id(repository, &span.file),
        start_line: span.start_line,
        start_column: span.start_column,
        end_line: span.end_line,
        end_column: span.end_column,
    }
}
fn origin(relationship: &Relationship) -> &str {
    match relationship {
        Relationship::Imports { origin, .. }
        | Relationship::References { origin, .. }
        | Relationship::Calls { origin, .. }
        | Relationship::Contains { origin, .. }
        | Relationship::DependsOn { origin, .. }
        | Relationship::Handles { origin, .. }
        | Relationship::Persists { origin, .. }
        | Relationship::Transforms { origin, .. }
        | Relationship::Consumes { origin, .. }
        | Relationship::Produces { origin, .. }
        | Relationship::Dispatches { origin, .. }
        | Relationship::InvokesPrompt { origin, .. } => origin,
    }
}
fn rank(origin: &str) -> u8 {
    match origin {
        "resolved" => 0,
        "partial" => 1,
        _ => 2,
    }
}
fn make_relationship(
    kind: &str,
    source: EntityId,
    target: EntityId,
    origin: String,
    evidence_id: Option<String>,
) -> Relationship {
    let id = v2::relationship_id(&source, &target, kind, &origin);
    macro_rules! rel {
        ($variant:ident) => {
            Relationship::$variant {
                id,
                source,
                target,
                origin,
                evidence_id,
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
