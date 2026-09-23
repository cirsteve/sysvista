use std::{collections::HashMap, path::Path};

use sysvista_cli::{
    discovery::Config,
    output::v2::{Evidence, Relationship},
    scanner,
};

/// The heuristic detectors infer `persists` from a model name appearing in a component
/// body, even in a string. That inference is weak, so the merged snapshot must keep it
/// labelled as a low-confidence heuristic and never present it as resolved.
#[test]
fn model_name_persists_stay_low_confidence_heuristics_after_merge() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/heuristic");
    let snapshot = scanner::scan_v2(&root, &Config::default()).unwrap();
    let evidence_by_id: HashMap<_, _> = snapshot
        .evidence
        .iter()
        .map(|evidence| match evidence {
            Evidence::Source { id, .. }
            | Evidence::SourceSnapshot { id, .. }
            | Evidence::Text { id, .. }
            | Evidence::Analyzer { id, .. } => (id.as_str(), evidence),
        })
        .collect();
    let persists: Vec<_> = snapshot
        .relationships
        .iter()
        .filter_map(|relationship| match relationship {
            Relationship::Persists { origin, evidence_id, .. } => Some((origin, evidence_id)),
            _ => None,
        })
        .collect();
    // service.py mentions `User` only in a string; the detector still matches it.
    assert_eq!(persists.len(), 1, "{persists:#?}");
    let (origin, evidence_id) = persists[0];
    assert_eq!(origin, "heuristic");
    let evidence = evidence_id
        .as_deref()
        .and_then(|id| evidence_by_id.get(id))
        .expect("persists keeps its evidence through the merge");
    assert!(
        matches!(evidence, Evidence::Analyzer { origin, rule, confidence, .. }
            if origin.as_deref() == Some("heuristic")
                && rule.as_deref() == Some("model_name_match")
                && confidence.as_deref() == Some("low")),
        "{evidence:#?}"
    );
}

/// Using a model as a type in TypeScript, with no component around it, creates no
/// persists edge from either stage.
#[test]
fn type_only_model_use_does_not_imply_persistence() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../corpus/cases/unrelated-model-references");
    let snapshot = scanner::scan_v2(&root, &Config::default()).unwrap();
    assert!(
        snapshot.entities.iter().any(|entity| entity.name == "labelFor"),
        "the formatter must be analyzed for this to mean anything"
    );
    assert!(
        !snapshot
            .relationships
            .iter()
            .any(|relationship| matches!(relationship, Relationship::Persists { .. }))
    );
}
