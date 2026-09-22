use std::path::Path;

use sysvista_cli::{
    discovery::{Config, Inventory},
    heuristic,
    output::v2::{Evidence, Relationship},
};

fn relationship_origin(relationship: &Relationship) -> &str {
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
fn adapter_labels_every_legacy_edge_as_heuristic() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/heuristic");
    let inventory = Inventory::discover(&root, &Config::default()).unwrap();
    let analysis = heuristic::analyze(&root, "fixture/heuristic", &inventory, &Config::default());

    assert!(!analysis.relationships.is_empty());
    assert!(
        analysis
            .relationships
            .iter()
            .all(|edge| relationship_origin(edge) == "heuristic")
    );

    let persists_evidence: Vec<_> = analysis
        .evidence
        .iter()
        .filter_map(|evidence| match evidence {
            Evidence::Analyzer {
                origin,
                confidence,
                rule,
                ..
            } if rule.as_deref() == Some("model_name_match") => Some((origin, confidence)),
            _ => None,
        })
        .collect();
    assert_eq!(persists_evidence.len(), 1);
    assert_eq!(persists_evidence[0].0.as_deref(), Some("heuristic"));
    assert_eq!(persists_evidence[0].1.as_deref(), Some("low"));
}

#[test]
fn traversal_claim_has_sets_without_order_or_steps() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/heuristic_workflow");
    let inventory = Inventory::discover(&root, &Config::default()).unwrap();
    let analysis = heuristic::analyze(&root, "fixture/heuristic", &inventory, &Config::default());
    let traversal_claims: Vec<_> = analysis
        .claims
        .iter()
        .filter(|claim| claim.predicate == "HeuristicTraversal")
        .collect();
    assert!(
        !traversal_claims.is_empty(),
        "fixture must exercise workflow traversal"
    );
    let emitted_relationship_ids: std::collections::HashSet<_> = analysis
        .relationships
        .iter()
        .map(|relationship| {
            serde_json::to_value(relationship).unwrap()["id"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    for claim in traversal_claims {
        let json = serde_json::to_string(claim).unwrap();
        assert!(!json.contains("\"order\""));
        assert!(!json.contains("\"steps\""));
        assert!(json.contains("entity_ids"));
        assert!(json.contains("relationship_ids"));
        for id in claim.object["relationship_ids"].as_array().unwrap() {
            assert!(emitted_relationship_ids.contains(id.as_str().unwrap()));
        }
    }
}
