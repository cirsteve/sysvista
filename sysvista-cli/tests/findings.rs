use std::path::Path;
use sysvista_cli::{discovery::Config, findings, output::v2::{self, Finding, Relationship, ScopeIndex}, scanner};

#[test]
fn findings_are_stable_and_cycle_uses_resolved_imports() {
    let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/findings");
    let mut snapshot=scanner::scan_v2(&root,&Config::load(&root).unwrap()).unwrap();
    let modules:Vec<_>=["a.ts","b.ts","c.ts"].iter().map(|path| snapshot.entities.iter().find(|e| e.name=="<module>" && snapshot.source_files.iter().any(|f| f.path==*path && f.id==e.file_id)).unwrap().id.clone()).collect();
    for (source,target) in [(0,1),(1,2),(2,0)] {
        snapshot.relationships.push(Relationship::Imports { id:v2::relationship_id(&modules[source],&modules[target],"imports","resolved"), source:modules[source].clone(), target:modules[target].clone(), origin:"resolved".into(), evidence_id:None });
    }
    snapshot.findings=findings::derive(&snapshot);
    let cycles:Vec<_>=snapshot.findings.iter().filter(|f|matches!(f,Finding::Rule{rule_id,..} if rule_id=="import_cycle")).collect();
    assert_eq!(cycles.len(),1);
    if let Finding::Rule{affected_file_ids,navigation_target,..}=cycles[0] { assert_eq!(affected_file_ids.len(),3); assert!(ScopeIndex::from_snapshot(&snapshot).scopes.iter().any(|s|s.scope_id==navigation_target.scope_id)); }
    assert!(snapshot.findings.iter().any(|f|matches!(f,Finding::Rule{rule_id,relationship_ids,..} if rule_id=="forbidden_dependency" && relationship_ids.len()==1)));
    assert_eq!(serde_json::to_vec(&snapshot.findings).unwrap(),serde_json::to_vec(&findings::derive(&snapshot)).unwrap());
}
