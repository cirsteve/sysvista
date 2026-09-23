use std::path::Path;
use std::collections::{BTreeMap, BTreeSet};
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
    assert!(ui.file_ids.iter().all(|id| index.scopes.iter()
        .find(|scope| scope.scope_id == ui.scope_id).unwrap().children.iter()
        .any(|child| matches!(child, sysvista_cli::output::v2::ScopeChild::File { file_id, .. } if file_id == id))));
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

#[test]
fn extension_filter_keeps_source_files_but_hides_children() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hierarchy");
    let mut config = Config::load(&root).unwrap();
    config.viewer.visible_extensions = vec!["ts".into()];
    let snapshot = scanner::scan_v2(&root, &config).unwrap();
    let file = snapshot.source_files.iter().find(|file| file.path == "src/ui/view.rs").unwrap();
    let index = snapshot.scope_index.as_ref().unwrap();
    assert!(index.scopes.iter().all(|scope| scope.children.iter().all(|child|
        !matches!(child, sysvista_cli::output::v2::ScopeChild::File { file_id, .. } if file_id == &file.id))));
}

#[test]
fn nested_declarations_use_their_owner_scope() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/analyzer_merge");
    let snapshot = scanner::scan_v2(&root, &Config::default()).unwrap();
    let owners: BTreeMap<_, _> = snapshot.entities.iter().map(|entity| (&entity.id, entity)).collect();
    let nested: Vec<_> = snapshot.entities.iter().filter(|entity| entity.owner_id.as_ref()
        .and_then(|id| owners.get(id)).is_some_and(|owner| owner.name != "<module>")).collect();
    assert!(!nested.is_empty());
    for entity in nested {
        let owner = owners[entity.owner_id.as_ref().unwrap()];
        let expected = sysvista_cli::output::v2::ScopeId(sysvista_cli::output::v2::stable_id("scope", &["symbol", owner.id.as_ref()]));
        assert_eq!(entity.scope_id, expected);
    }
    let index = snapshot.scope_index.as_ref().unwrap();
    let root_slice = index.scopes.iter().find(|scope| scope.scope_id == snapshot.manifest.root_scope_id).unwrap();
    for module in snapshot.entities.iter().filter(|entity| entity.name == "<module>") {
        assert_eq!(root_slice.owner_map.get(module.id.as_ref()), Some(&module.file_id.0));
        assert!(!root_slice.children.iter().any(|child| matches!(child,
            sysvista_cli::output::v2::ScopeChild::Symbol { entity_id, .. } if entity_id == &module.id)));
    }
}

#[test]
fn physical_containment_is_acyclic_and_packages_own_immediate_files() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hierarchy");
    let snapshot = scanner::scan_v2(&root, &Config::load(&root).unwrap()).unwrap();
    let inventory = Inventory { entries: vec![InventoryEntry { path: "src/ui/Cargo.toml".into(), outcome: InventoryOutcome::Included }] };
    let projections = hierarchy::physical::derive(&snapshot.manifest.repository, &inventory, &snapshot.source_files, &snapshot.entities);
    let physical: BTreeMap<_, _> = projections.iter()
        .filter(|p| p.kind != "module")
        .map(|p| (p.scope_id.clone(), p))
        .collect();
    for projection in physical.values() {
        let mut visited = BTreeSet::new();
        let mut cursor = Some(projection.scope_id.clone());
        while let Some(id) = cursor {
            assert!(visited.insert(id.clone()), "containment cycle at {id:?}");
            cursor = physical.get(&id).and_then(|p| p.parent_scope_id.clone());
        }
    }
    for package in physical.values().filter(|p| p.kind == "package") {
        for file in physical.values().filter(|p| p.kind == "file") {
            if Path::new(&file.name).parent().is_some_and(|parent| parent.to_string_lossy() == package.name) {
                assert_eq!(file.parent_scope_id.as_ref(), Some(&package.scope_id));
            }
        }
    }
    assert!(physical.values().any(|p| p.kind == "package" && p.name == "src/ui"));
    assert!(!physical.values().any(|p| p.kind == "directory" && p.name == "src/ui"));

    let root_package = Inventory { entries: vec![InventoryEntry { path: "package.json".into(), outcome: InventoryOutcome::Included },
        InventoryEntry { path: "src/ui/Cargo.toml".into(), outcome: InventoryOutcome::Included }] };
    let rooted = hierarchy::physical::derive(&snapshot.manifest.repository, &root_package, &snapshot.source_files, &snapshot.entities);
    let package = rooted.iter().find(|p| p.kind == "package" && p.parent_scope_id.as_ref() == Some(&snapshot.manifest.root_scope_id)).unwrap();
    let src = rooted.iter().find(|p| p.kind == "directory" && p.name == "src").unwrap();
    assert_eq!(src.parent_scope_id.as_ref(), Some(&package.scope_id));
}
