use std::path::Path;
use sysvista_cli::{
    discovery::{Config, Inventory, InventoryEntry, InventoryOutcome},
    hierarchy,
    output::v2::{Diagnostic, Projection, ScopeIndex},
    scanner,
};

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
    let directory = snapshot
        .projections
        .iter()
        .find(|projection| projection.kind == "directory" && projection.name == "src/ui")
        .unwrap();
    let file = snapshot
        .projections
        .iter()
        .find(|projection| projection.kind == "file" && projection.name == "src/ui/view.rs")
        .unwrap();
    assert_eq!(file.parent_scope_id.as_ref(), Some(&directory.scope_id));
    assert!(!directory.entity_ids.is_empty());

    let index = ScopeIndex::from_snapshot(&snapshot);
    let directory_slice = index
        .scopes
        .iter()
        .find(|scope| scope.scope_id == directory.scope_id)
        .unwrap();
    assert!(directory_slice.child_scope_ids.contains(&file.scope_id));
    assert!(ui.entity_ids.iter().all(|id| {
        index
            .scopes
            .iter()
            .find(|scope| scope.scope_id == ui.scope_id)
            .unwrap()
            .child_ids
            .contains(&id.0)
    }));
}

#[test]
fn excluded_package_markers_do_not_create_package_projections() {
    let inventory = Inventory {
        entries: vec![InventoryEntry {
            path: "vendor/package.json".into(),
            outcome: InventoryOutcome::Excluded {
                rule: "vendor/**".into(),
            },
        }],
    };
    let projections = hierarchy::physical::derive("repository", &inventory, &[], &[]);
    assert!(
        projections
            .iter()
            .all(|projection| projection.kind != "package")
    );
}

#[test]
fn legacy_projection_without_kind_still_deserializes() {
    let projection: Projection = serde_json::from_value(serde_json::json!({
        "id": "projection:legacy",
        "name": "legacy",
        "scope_id": "scope:legacy",
        "entity_ids": [],
        "relationship_ids": []
    }))
    .unwrap();
    assert_eq!(projection.kind, "legacy");
}
