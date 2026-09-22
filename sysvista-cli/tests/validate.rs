use std::path::Path;
use sysvista_cli::{
    discovery::Config,
    output::v2::{self, Diagnostic, EntityId, Relationship, RelationshipId},
    scanner, validate,
};

#[test]
fn scanned_snapshot_has_complete_reference_and_membership_coverage() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hierarchy");
    let snapshot = scanner::scan_v2(&root, &Config::load(&root).unwrap()).unwrap();
    let diagnostics = validate::validate(&snapshot);
    assert!(!diagnostics.iter().any(|d| matches!(
        d,
        Diagnostic::DanglingReference { .. } | Diagnostic::Coverage { .. }
    )));
}

#[test]
fn dangling_relationship_target_is_reported_with_relationship_id() {
    let mut snapshot = fixture_snapshot();
    let source = snapshot.entities[0].id.clone();
    let missing = EntityId("entity:missing".into());
    let relationship_id = RelationshipId("relationship:dangling".into());
    snapshot.relationships.push(Relationship::Imports {
        id: relationship_id.clone(),
        source,
        target: missing,
        origin: "resolved".into(),
        evidence_id: None,
    });

    let diagnostics = validate::validate(&snapshot);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::DanglingReference { relationship_id: actual, .. } if actual == &relationship_id
    )));
}

#[test]
fn resolved_and_heuristic_disagreement_is_reported_and_retained() {
    let mut snapshot = fixture_snapshot();
    let source = snapshot.entities[0].id.clone();
    let resolved_target = snapshot.entities[1].id.clone();
    let heuristic_target = snapshot.entities[2].id.clone();
    let resolved_id = v2::relationship_id(&source, &resolved_target, "imports", "resolved");
    let heuristic_id = v2::relationship_id(&source, &heuristic_target, "imports", "heuristic");
    snapshot.relationships.extend([
        Relationship::Imports {
            id: resolved_id.clone(),
            source: source.clone(),
            target: resolved_target,
            origin: "resolved".into(),
            evidence_id: None,
        },
        Relationship::Imports {
            id: heuristic_id.clone(),
            source,
            target: heuristic_target,
            origin: "heuristic".into(),
            evidence_id: None,
        },
    ]);

    let diagnostics = validate::validate(&snapshot);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::Contradiction { relationship_ids, .. }
            if relationship_ids.contains(&resolved_id) && relationship_ids.contains(&heuristic_id)
    )));
    assert!(
        snapshot
            .relationships
            .iter()
            .any(|relationship| relationship.sort_key().0 == &resolved_id)
    );
    assert!(
        snapshot
            .relationships
            .iter()
            .any(|relationship| relationship.sort_key().0 == &heuristic_id)
    );
}

fn fixture_snapshot() -> sysvista_cli::output::v2::Snapshot {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hierarchy");
    scanner::scan_v2(&root, &Config::load(&root).unwrap()).unwrap()
}
