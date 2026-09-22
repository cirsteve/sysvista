use std::{collections::HashMap, fs, path::Path};

use sysvista_cli::{
    discovery::Config,
    output::v2::{Evidence, Relationship},
    scanner,
};

#[test]
fn unrelated_model_reference_does_not_imply_persistence() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../corpus/cases/unrelated-model-references");
    let expected: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("expected.json")).unwrap()).unwrap();
    assert_eq!(expected["relationships"]["persists"], serde_json::json!([]));

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
        .filter(|relationship| matches!(relationship, Relationship::Persists { .. }))
        .collect();
    assert!(
        persists.is_empty(),
        "the case explicitly labels every persists relationship as a false positive: {persists:#?}"
    );

    let model_name_persists: Vec<_> = snapshot
        .relationships
        .iter()
        .filter_map(|relationship| match relationship {
            Relationship::Persists {
                evidence_id: Some(evidence_id),
                ..
            } => match evidence_by_id.get(evidence_id.as_str()) {
                Some(Evidence::Analyzer { origin, rule, .. })
                    if origin.as_deref() == Some("heuristic")
                        && rule.as_deref() == Some("model_name_match") =>
                {
                    Some(relationship)
                }
                _ => None,
            },
            _ => None,
        })
        .collect();

    assert!(
        model_name_persists.is_empty(),
        "a model name alone must not create a persists relationship: {model_name_persists:#?}"
    );
}
