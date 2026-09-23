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
fn resolved_and_heuristic_edges_from_one_source_are_not_errors() {
    // One module importing three targets, one resolved and two heuristic, is ordinary
    // fan-out; it used to be reported as quadratic contradictions.
    let mut snapshot = fixture_snapshot();
    let source = snapshot.entities[0].id.clone();
    let targets: Vec<_> = snapshot.entities[1..4].iter().map(|entity| entity.id.clone()).collect();
    for (target, origin) in targets.iter().zip(["resolved", "heuristic", "heuristic"]) {
        snapshot.relationships.push(Relationship::Imports {
            id: v2::relationship_id(&source, target, "imports", origin),
            source: source.clone(),
            target: target.clone(),
            origin: origin.into(),
            evidence_id: None,
        });
    }
    let diagnostics = validate::validate(&snapshot);
    assert_eq!(validate::summary(&diagnostics).errors, 0, "{diagnostics:#?}");
}

fn fixture_snapshot() -> sysvista_cli::output::v2::Snapshot {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hierarchy");
    scanner::scan_v2(&root, &Config::load(&root).unwrap()).unwrap()
}
