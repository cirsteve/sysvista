use std::{
    fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use sysvista_cli::{discovery::Config, output::v2, scanner};

#[test]
fn a_shared_origin_keeps_entity_ids_unique_and_equal_across_roots() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!(
        "sysvista-unique-roots-{}-{nonce}",
        std::process::id()
    ));
    let mut scans = Vec::new();
    for name in ["first", "second"] {
        let root = base.join(name);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("unit.ts"), "export function first() { const same = 1; }\nexport function second() { const same = 2; }\n").unwrap();
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .current_dir(&root)
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .args([
                    "remote",
                    "add",
                    "origin",
                    "https://example.com/team/identity.git"
                ])
                .current_dir(&root)
                .status()
                .unwrap()
                .success()
        );
        let snapshot = scanner::scan_v2(&root, &Config::default()).unwrap();
        assert!(v2::duplicate_ids(&snapshot).is_empty());
        scans.push(snapshot);
    }
    assert_eq!(scans[0].manifest.repository, scans[1].manifest.repository);
    assert_eq!(
        scans[0]
            .entities
            .iter()
            .map(|entity| &entity.id)
            .collect::<Vec<_>>(),
        scans[1]
            .entities
            .iter()
            .map(|entity| &entity.id)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        scans[0]
            .entities
            .iter()
            .filter(|entity| entity.name == "same" && entity.is_local)
            .count(),
        2
    );
    fs::remove_dir_all(base).unwrap();
}

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
        let Ok(config) = Config::load(&root) else {
            continue;
        };
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
    let conflicting = v2::Claim {
        object: serde_json::json!({"target": "b"}),
        ..claim.clone()
    };
    snapshot.claims = vec![claim.clone(), claim.clone(), conflicting];
    // Collections other than evidence and claims keep their first record too.
    let entities = snapshot.entities.len();
    let renamed = v2::CodeEntity {
        name: "renamed".into(),
        ..snapshot.entities[0].clone()
    };
    snapshot.entities.push(renamed);
    let diagnostics = v2::dedup_records(&mut snapshot);
    assert_eq!(snapshot.claims.len(), 1);
    assert_eq!(snapshot.entities.len(), entities);
    assert_ne!(snapshot.entities[0].name, "renamed");
    assert_eq!(diagnostics.len(), 2, "{diagnostics:#?}");
    assert!(v2::duplicate_ids(&snapshot).is_empty());
}
