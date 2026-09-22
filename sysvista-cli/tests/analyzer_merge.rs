use sysvista_cli::{
    analyzer::{
        self,
        contract::{AnalyzerEntity, AnalyzerRelationship},
    },
    heuristic::HeuristicAnalysis,
    output::v2::{self, Relationship},
};

fn entity(file: &str, name: &str) -> AnalyzerEntity {
    AnalyzerEntity {
        name: name.into(),
        ownership_chain: name.into(),
        declaration_kind: "function".into(),
        file: file.into(),
        discriminator: 0,
        start_line: 1,
        start_column: 1,
        end_line: 1,
        end_column: 10,
        attributes: Default::default(),
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

#[test]
fn duplicate_names_use_file_scoped_ids_and_resolved_is_primary() {
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
    let response = analyzer::AnalyzeResponse {
        contract_version: 1,
        analyzer_version: "test".into(),
        entities: vec![source, target],
        relationships: vec![AnalyzerRelationship {
            kind: "calls".into(),
            source: "a.ts#duplicate#function#0".into(),
            target: Some("b.ts#duplicate#function#0".into()),
            origin: "resolved".into(),
            name: None,
            span: None,
        }],
        unresolved: vec![],
        diagnostics: vec![],
        payloads: vec![],
    };
    let merged = analyzer::merge("repo", response, heuristic);
    assert_ne!(merged.entities[0].id, merged.entities[1].id);
    assert_eq!(merged.relationships.len(), 2);
    assert_eq!(origin(&merged.relationships[0]), "resolved");
    assert_eq!(origin(&merged.relationships[1]), "heuristic");
}
