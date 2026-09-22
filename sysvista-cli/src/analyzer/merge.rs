use std::collections::{BTreeMap, HashMap};

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

pub fn merge(
    repository: &str,
    response: AnalyzeResponse,
    heuristic: HeuristicAnalysis,
) -> MergedAnalysis {
    let mut key_to_id = HashMap::new();
    for entity in &response.entities {
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
    let mut entities = heuristic.entities;
    for entity in &response.entities {
        let file_id = v2::file_id(repository, &entity.file);
        let id = key_to_id[&key(entity)].clone();
        let parent = entity
            .ownership_chain
            .rsplit_once('.')
            .map(|(value, _)| value);
        let owner_id = parent
            .and_then(|parent| {
                response.entities.iter().find(|candidate| {
                    candidate.file == entity.file && candidate.ownership_chain == parent
                })
            })
            .and_then(|owner| key_to_id.get(&key(owner)))
            .cloned();
        entities.push(CodeEntity {
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
    entities.sort_by(|a, b| a.id.cmp(&b.id));
    entities.dedup_by(|a, b| a.id == b.id);
    let mut relationships = heuristic.relationships;
    let mut evidence = heuristic.evidence;
    for edge in response
        .relationships
        .iter()
        .filter(|edge| edge.target.is_some())
    {
        let (Some(source), Some(target)) = (
            key_to_id.get(&edge.source),
            edge.target
                .as_ref()
                .and_then(|target| key_to_id.get(target)),
        ) else {
            continue;
        };
        let evidence_id = v2::stable_id(
            "evidence",
            &[&edge.origin, source.as_ref(), target.as_ref(), &edge.kind],
        );
        evidence.push(Evidence::Analyzer {
            id: evidence_id.clone(),
            analyzer: response.analyzer_version.clone(),
            detail: edge.name.clone().unwrap_or_else(|| edge.kind.clone()),
            origin: Some(edge.origin.clone()),
            confidence: Some(
                if edge.origin == "resolved" {
                    "high"
                } else {
                    "low"
                }
                .into(),
            ),
            rule: None,
        });
        relationships.push(make_relationship(
            &edge.kind,
            source.clone(),
            target.clone(),
            edge.origin.clone(),
            Some(evidence_id),
        ));
    }
    relationships
        .sort_by(|a, b| (rank(origin(a)), a.sort_key()).cmp(&(rank(origin(b)), b.sort_key())));
    relationships.dedup_by(|a, b| a.sort_key() == b.sort_key());
    let unresolved = response
        .unresolved
        .into_iter()
        .filter_map(|item| {
            Some(UnresolvedReference {
                name: item.name,
                source: key_to_id.get(&item.source)?.clone(),
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
                .filter_map(|key| key_to_id.get(key).cloned())
                .collect(),
            consumer_ids: payload
                .consumers
                .iter()
                .filter_map(|key| key_to_id.get(key).cloned())
                .collect(),
        })
        .collect();
    let mut diagnostics = heuristic.diagnostics;
    diagnostics.extend(response.diagnostics.into_iter().map(|diagnostic| {
        Diagnostic::AnalyzerIssue {
            message: diagnostic.message,
            severity: diagnostic.severity,
            span: diagnostic.span.map(|span| convert_span(repository, span)),
        }
    }));
    MergedAnalysis {
        entities,
        relationships,
        unresolved,
        evidence,
        claims: heuristic.claims,
        payloads,
        diagnostics,
    }
}

fn key(entity: &super::contract::AnalyzerEntity) -> String {
    format!(
        "{}#{}#{}#{}",
        entity.file, entity.ownership_chain, entity.declaration_kind, entity.discriminator
    )
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
