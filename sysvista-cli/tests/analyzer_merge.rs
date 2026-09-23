use std::{collections::BTreeSet, path::Path};
use sysvista_cli::{
    analyzer::{
        self,
        contract::{AnalyzerDiagnostic, AnalyzerEntity, AnalyzerPayload, AnalyzerRelationship, AnalyzerSpan},
    },
    heuristic::HeuristicAnalysis,
    output::v2::{self, Diagnostic, Evidence, Relationship},
};
use sysvista_cli::{discovery::Config, scanner};

fn entity(file: &str, name: &str) -> AnalyzerEntity {
    AnalyzerEntity {
        name: name.into(),
        ownership_chain: name.into(),
        declaration_kind: "function".into(),
        file: file.into(),
        discriminator: 0,
        owner_key: None,
        start_line: 1,
        start_column: 1,
        end_line: 1,
        end_column: 10,
        attributes: Default::default(),
    }
}

#[test]
fn on_disk_cross_stage_import_has_one_identity_and_both_origins() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/analyzer_merge");
    let snapshot = scanner::scan_v2(&root, &Config::default()).unwrap();
    let source_file_id = snapshot
        .source_files
        .iter()
        .find(|file| file.path == "source.ts")
        .unwrap()
        .id
        .clone();
    let module = snapshot
        .entities
        .iter()
        .find(|entity| entity.name == "<module>" && entity.file_id == source_file_id)
        .unwrap();
    let target_entities: Vec<_> = snapshot
        .entities
        .iter()
        .filter(|entity| entity.name == "Target")
        .collect();
    assert_eq!(
        target_entities.len(),
        1,
        "heuristic and resolved declarations must be reconciled"
    );
    let target = target_entities[0];
    let origins: Vec<_> = snapshot
        .relationships
        .iter()
        .filter_map(|relationship| match relationship {
            Relationship::Imports {
                source,
                target: relationship_target,
                origin,
                ..
            } if source == &module.id && relationship_target == &target.id => Some(origin.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(origins, vec!["resolved", "heuristic"]);
    assert!(snapshot.diagnostics.iter().any(|diagnostic| matches!(diagnostic,
        Diagnostic::PayloadIdentityConflict { name, .. } if name == "Payload")));
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

fn context(files: &BTreeSet<String>) -> analyzer::MergeContext<'_> {
    analyzer::MergeContext {
        repository: "repo",
        root: "/checkout/repo",
        files,
    }
}

fn files(paths: &[&str]) -> BTreeSet<String> {
    paths.iter().map(|path| (*path).to_owned()).collect()
}

fn response(entities: Vec<AnalyzerEntity>, relationships: Vec<AnalyzerRelationship>) -> analyzer::AnalyzeResponse {
    analyzer::AnalyzeResponse {
        contract_version: analyzer::CONTRACT_VERSION,
        analyzer_version: "test".into(),
        entities,
        relationships,
        unresolved: vec![],
        diagnostics: vec![],
        payloads: vec![],
    }
}

#[test]
fn same_payload_name_in_distinct_files_is_a_diagnostic() {
    let mut first = entity("source.ts", "Payload");
    first.declaration_kind = "interface".into();
    let mut second = entity("target.ts", "Payload");
    second.declaration_kind = "interface".into();
    let mut response = response(vec![first, second], vec![]);
    response.payloads = vec![AnalyzerPayload { name: "Payload".into(), producers: vec![], consumers: vec![] }];
    let files = files(&["source.ts", "target.ts"]);
    let merged = analyzer::merge(&context(&files), response, HeuristicAnalysis::default());
    assert_eq!(merged.payloads.len(), 1);
    assert!(merged.diagnostics.iter().any(|diagnostic| matches!(diagnostic,
        Diagnostic::PayloadIdentityConflict { name, file_ids, .. } if name == "Payload" && file_ids.len() == 2)));
}

fn span(file: &str, line: u32) -> AnalyzerSpan {
    AnalyzerSpan {
        file: file.into(),
        start_line: line,
        start_column: 1,
        end_line: line,
        end_column: 10,
    }
}

#[test]
fn resolved_and_heuristic_edges_are_both_kept_with_resolved_first() {
    let source = entity("a.ts", "duplicate");
    let target = entity("b.ts", "duplicate");
    let source_id = v2::entity_id(&v2::file_id("repo", "a.ts"), "duplicate", "function", 0);
    let target_id = v2::entity_id(&v2::file_id("repo", "b.ts"), "duplicate", "function", 0);
    let heuristic_relationship = Relationship::Calls {
        id: v2::relationship_id(&source_id, &target_id, "calls", "heuristic"),
        source: source_id,
        target: target_id,
        origin: "heuristic".into(),
        evidence_id: None,
    };
    let heuristic = HeuristicAnalysis {
        relationships: vec![heuristic_relationship],
        ..Default::default()
    };
    let known = files(&["a.ts", "b.ts"]);
    let merged = analyzer::merge(
        &context(&known),
        response(
            vec![source, target],
            vec![AnalyzerRelationship {
                kind: "calls".into(),
                source: "a.ts#duplicate#function#0".into(),
                target: Some("b.ts#duplicate#function#0".into()),
                origin: "resolved".into(),
                name: None,
                span: None,
            }],
        ),
        heuristic,
    );
    assert_ne!(merged.entities[0].id, merged.entities[1].id);
    assert_eq!(merged.relationships.len(), 2);
    assert_eq!(origin(&merged.relationships[0]), "resolved");
    assert_eq!(origin(&merged.relationships[1]), "heuristic");
}

#[test]
fn discriminators_keep_same_named_declarations_distinct_and_owners_exact() {
    // Two overload signatures and an implementation share file, chain and kind.
    // A declaration nested in the implementation must be owned by the implementation,
    // not by the first overload that happens to share its parent chain.
    let mut overloads: Vec<_> = (0..3)
        .map(|discriminator| AnalyzerEntity {
            discriminator,
            start_line: 1 + discriminator as u32,
            ..entity("math.ts", "parse")
        })
        .collect();
    overloads.push(AnalyzerEntity {
        name: "helper".into(),
        ownership_chain: "parse.helper".into(),
        declaration_kind: "variable".into(),
        owner_key: Some("math.ts#parse#function#2".into()),
        start_line: 4,
        ..entity("math.ts", "helper")
    });
    let known = files(&["math.ts"]);
    let merged = analyzer::merge(&context(&known), response(overloads, vec![]), HeuristicAnalysis::default());
    let file = v2::file_id("repo", "math.ts");
    let ids: BTreeSet<_> = (0..3).map(|d| v2::entity_id(&file, "parse", "function", d)).collect();
    assert_eq!(ids.len(), 3);
    assert!(ids.iter().all(|id| merged.entities.iter().any(|entity| &entity.id == id)));
    let helper = merged.entities.iter().find(|entity| entity.name == "helper").unwrap();
    assert_eq!(helper.owner_id, Some(v2::entity_id(&file, "parse", "function", 2)));
    assert!(merged.diagnostics.is_empty(), "{:?}", merged.diagnostics);
}

#[test]
fn repeated_call_sites_share_one_evidence_record_with_every_site() {
    let known = files(&["a.ts", "b.ts"]);
    let call = |line| AnalyzerRelationship {
        kind: "calls".into(),
        source: "a.ts#caller#function#0".into(),
        target: Some("b.ts#callee#function#0".into()),
        origin: "resolved".into(),
        name: Some(if line == 2 { "callee" } else { "lib.callee" }.into()),
        span: Some(span("a.ts", line)),
    };
    let merged = analyzer::merge(
        &context(&known),
        response(vec![entity("a.ts", "caller"), entity("b.ts", "callee")], vec![call(2), call(5)]),
        HeuristicAnalysis::default(),
    );
    assert_eq!(merged.relationships.len(), 1);
    assert_eq!(merged.evidence.len(), 1);
    let Evidence::Analyzer { sites, detail, .. } = &merged.evidence[0] else {
        panic!("analyzer evidence expected");
    };
    assert_eq!(sites.iter().map(|site| site.start_line).collect::<Vec<_>>(), vec![2, 5]);
    assert_eq!(detail, "callee, lib.callee");
}

#[test]
fn unmatched_keys_and_outside_files_are_reported_not_silently_dropped() {
    let known = files(&["a.ts"]);
    let mut outside = entity("/elsewhere/lib.ts", "external");
    outside.file = "/elsewhere/lib.ts".into();
    let merged = analyzer::merge(
        &context(&known),
        response(
            vec![entity("a.ts", "caller"), outside],
            vec![
                AnalyzerRelationship {
                    kind: "calls".into(),
                    source: "a.ts#caller#function#0".into(),
                    target: Some("a.ts#missing#function#0".into()),
                    origin: "resolved".into(),
                    name: None,
                    span: None,
                },
                // The dropped entity's key is unmatched, and its file is a host path.
                AnalyzerRelationship {
                    kind: "calls".into(),
                    source: "a.ts#caller#function#0".into(),
                    target: Some("/elsewhere/lib.ts#external#function#0".into()),
                    origin: "resolved".into(),
                    name: None,
                    span: None,
                },
            ],
        ),
        HeuristicAnalysis::default(),
    );
    assert!(merged.relationships.is_empty());
    assert_eq!(merged.entities.len(), 1, "the out-of-inventory entity is dropped");
    let messages: Vec<_> = merged
        .diagnostics
        .iter()
        .filter_map(|diagnostic| match diagnostic {
            Diagnostic::Warning { message, .. } => Some(message.as_str()),
            _ => None,
        })
        .collect();
    assert!(messages.iter().any(|message| message.contains("a.ts#missing#function#0")));
    assert!(messages.iter().any(|message| message.contains("<outside scan>#external#function#0")));
    assert!(messages.iter().any(|message| message.contains("1 files outside the scan inventory")));
    assert!(!messages.iter().any(|message| message.contains("/elsewhere")), "host paths must not leak");
}

#[test]
fn analyzer_messages_do_not_carry_the_scan_root() {
    let known = files(&["a.ts"]);
    let mut input = response(vec![], vec![]);
    input.diagnostics.push(AnalyzerDiagnostic {
        message: "File '/checkout/repo/a.ts' is not listed; see /checkout/repo".into(),
        severity: "warning".into(),
        span: None,
    });
    let merged = analyzer::merge(&context(&known), input, HeuristicAnalysis::default());
    let Diagnostic::AnalyzerIssue { message, .. } = &merged.diagnostics[0] else {
        panic!("analyzer issue expected");
    };
    assert_eq!(message, "File 'a.ts' is not listed; see .");
}

#[test]
fn analyzer_keys_round_trip_between_the_node_and_rust_contracts() {
    // The key format is shared by `entityKey` in the analyzer and `key` in merge.rs.
    let known = files(&["src/a.ts"]);
    let entity = AnalyzerEntity {
        name: "run".into(),
        ownership_chain: "Service.run".into(),
        declaration_kind: "method".into(),
        file: "src/a.ts".into(),
        discriminator: 1,
        ..entity("src/a.ts", "run")
    };
    let payload = analyzer::contract::AnalyzerPayload {
        name: "Input".into(),
        producers: vec!["src/a.ts#Service.run#method#1".into()],
        consumers: vec![],
    };
    let mut input = response(vec![entity], vec![]);
    input.payloads.push(payload);
    let merged = analyzer::merge(&context(&known), input, HeuristicAnalysis::default());
    assert_eq!(
        merged.payloads[0].producer_ids,
        vec![v2::entity_id(&v2::file_id("repo", "src/a.ts"), "Service.run", "method", 1)]
    );
}
