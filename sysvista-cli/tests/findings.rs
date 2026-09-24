use std::path::Path;
use sysvista_cli::{
    discovery::Config,
    findings,
    output::v2::{Finding, ScopeIndex},
    scanner,
};

#[test]
fn findings_are_stable_and_cycle_uses_resolved_imports() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/findings");
    let snapshot = scanner::scan_v2(&root, &Config::load(&root).unwrap()).unwrap();
    let cycles: Vec<_> = snapshot
        .findings
        .iter()
        .filter(|f| matches!(f,Finding::Rule{rule_id,..} if rule_id=="import_cycle"))
        .collect();
    assert_eq!(cycles.len(), 1);
    if let Finding::Rule {
        affected_file_ids,
        navigation_target,
        ..
    } = cycles[0]
    {
        assert_eq!(affected_file_ids.len(), 3);
        assert!(
            ScopeIndex::from_snapshot(&snapshot)
                .scopes
                .iter()
                .any(|s| s.scope_id == navigation_target.scope_id)
        );
    }
    assert!(snapshot.findings.iter().any(|f|matches!(f,Finding::Rule{rule_id,relationship_ids,..} if rule_id=="forbidden_dependency" && relationship_ids.len()==1)));
    assert_eq!(
        serde_json::to_vec(&snapshot.findings).unwrap(),
        serde_json::to_vec(&findings::derive(&snapshot)).unwrap()
    );
}
