use std::path::Path;
use sysvista_cli::{discovery::Config, output::v2::Diagnostic, scanner};

#[test]
fn hierarchy_assigns_specific_owners_and_reports_equal_ties() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hierarchy");
    let snapshot = scanner::scan_v2(&root, &Config::load(&root).unwrap()).unwrap();
    let ui = snapshot.modules.iter().find(|m| m.name == "ui").unwrap();
    assert!(ui.file_ids.iter().any(|id| {
        snapshot
            .source_files
            .iter()
            .any(|f| &f.id == id && f.path == "src/ui/view.rs")
    }));
    let unassigned = snapshot
        .modules
        .iter()
        .find(|m| m.name == "Unassigned")
        .unwrap();
    let conflicted = snapshot
        .source_files
        .iter()
        .find(|f| f.path == "src/shared.rs")
        .unwrap();
    assert!(unassigned.file_ids.contains(&conflicted.id));
    assert_eq!(snapshot.diagnostics.iter().filter(|d| matches!(d, Diagnostic::MembershipConflict { file_id, .. } if file_id == &conflicted.id)).count(), 1);
    assert!(snapshot.projections.iter().all(|p| {
        snapshot
            .projections
            .iter()
            .any(|candidate| candidate.scope_id == p.scope_id)
    }));
}
