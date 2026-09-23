use std::{fs, path::Path};

use sysvista_cli::{discovery::Config, output::v2, scanner};

/// Every id collection in graph.json is indexed by id downstream, so a duplicate
/// silently loses a record. Scan every corpus case and fixture and require none.
#[test]
fn every_emitted_id_collection_is_unique() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut roots: Vec<_> = fs::read_dir(manifest.join("../corpus/cases"))
        .unwrap()
        .chain(fs::read_dir(manifest.join("tests/fixtures")).unwrap())
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();
    roots.sort();
    assert!(roots.len() > 20);
    for root in roots {
        let Ok(config) = Config::load(&root) else { continue };
        let snapshot = scanner::scan_v2(&root, &config).unwrap();
        let duplicates = v2::duplicate_ids(&snapshot);
        assert!(duplicates.is_empty(), "{}: {duplicates:#?}", root.display());
        let dropped: Vec<_> = snapshot
            .diagnostics
            .iter()
            .filter(|diagnostic| matches!(diagnostic, v2::Diagnostic::Warning { message, .. } if message.contains("was emitted for different records")))
            .collect();
        assert!(dropped.is_empty(), "{}: {dropped:#?}", root.display());
    }
}

#[test]
fn conflicting_records_for_one_id_are_reported_and_identical_ones_collapse() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/findings");
    let mut snapshot = scanner::scan_v2(&root, &Config::load(&root).unwrap()).unwrap();
    let claim = v2::Claim {
        id: "claim:duplicate".into(),
        subject: snapshot.entities[0].id.clone(),
        predicate: "rule".into(),
        object: serde_json::json!({"target": "a"}),
        evidence_ids: vec![],
    };
    let conflicting = v2::Claim { object: serde_json::json!({"target": "b"}), ..claim.clone() };
    snapshot.claims = vec![claim.clone(), claim.clone(), conflicting];
    let diagnostics = v2::dedup_records(&mut snapshot);
    assert_eq!(snapshot.claims.len(), 1);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert!(v2::duplicate_ids(&snapshot).is_empty());
}
