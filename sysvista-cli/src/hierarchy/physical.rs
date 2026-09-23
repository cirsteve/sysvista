use crate::{
    discovery::{Inventory, InventoryOutcome},
    output::v2::{self, CodeEntity, Projection, ScopeId, SourceFile},
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub fn derive(
    repository: &str,
    inventory: &Inventory,
    files: &[SourceFile],
    entities: &[CodeEntity],
) -> Vec<Projection> {
    let root = ScopeId(v2::stable_id("scope", &["repository", repository]));
    let mut out = vec![node(
        repository,
        "repository",
        root.clone(),
        None,
        entities.iter().map(|entity| entity.id.clone()).collect(),
    )];
    let file_by_path: BTreeMap<_, _> = files.iter().map(|f| (f.path.as_str(), f)).collect();
    let mut dirs = BTreeSet::new();
    for entry in inventory
        .entries
        .iter()
        .filter(|e| !matches!(e.outcome, InventoryOutcome::Excluded { .. }))
    {
        let mut parent = Path::new(&entry.path).parent();
        while let Some(path) = parent {
            if path.as_os_str().is_empty() {
                break;
            }
            dirs.insert(path.to_string_lossy().replace('\\', "/"));
            parent = path.parent();
        }
    }
    let mut packages = BTreeSet::new();
    for marker in ["package.json", "Cargo.toml", "pyproject.toml"] {
        for entry in inventory.entries.iter()
            .filter(|e| !matches!(e.outcome, InventoryOutcome::Excluded { .. }))
            .filter(|e| e.path == marker || e.path.ends_with(&format!("/{marker}"))) {
            let path = Path::new(&entry.path).parent().map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_default();
            packages.insert(path);
        }
    }
    let scope_for_path = |path: &str| {
        let kind = if packages.contains(path) { "package" } else { "directory" };
        ScopeId(v2::stable_id("scope", &[kind, repository, path]))
    };
    for dir in &dirs {
        let kind = if packages.contains(dir) { "package" } else { "directory" };
        let scope = scope_for_path(dir);
        let parent_path = Path::new(dir)
            .parent()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .filter(|p| !p.is_empty());
        let parent = parent_path
            .map(|p| scope_for_path(&p))
            .unwrap_or_else(|| root.clone());
        out.push(node(
            dir,
            kind,
            scope,
            Some(parent),
            entities_for_path(dir, files, entities),
        ));
    }
    if packages.contains("") {
        out.push(node(repository, "package", scope_for_path(""), Some(root.clone()),
            entities_for_path("", files, entities)));
    }
    for (path, file) in file_by_path {
        let parent_path = Path::new(path)
            .parent()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .filter(|p| !p.is_empty());
        let parent = parent_path.map(|p| scope_for_path(&p))
            .unwrap_or_else(|| if packages.contains("") { scope_for_path("") } else { root.clone() });
        let ids = entities
            .iter()
            .filter(|e| e.file_id == file.id)
            .map(|e| e.id.clone())
            .collect();
        out.push(node(
            path,
            "file",
            v2::scope_id(&file.id),
            Some(parent),
            ids,
        ));
    }
    let owners: BTreeSet<_> = entities.iter().filter_map(|entity| entity.owner_id.as_ref())
        .filter(|id| entities.iter().any(|e| &e.id == *id && e.name != "<module>"))
        .collect();
    for entity in entities.iter().filter(|entity| owners.contains(&entity.id)) {
        let scope = ScopeId(v2::stable_id("scope", &["symbol", entity.id.as_ref()]));
        out.push(node(
            &entity.qualified_name,
            "symbol",
            scope,
            Some(entity.scope_id.clone()),
            vec![entity.id.clone()],
        ));
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out.dedup_by(|a, b| a.scope_id == b.scope_id);
    out
}

fn entities_for_path(
    path: &str,
    files: &[SourceFile],
    entities: &[CodeEntity],
) -> Vec<crate::output::v2::EntityId> {
    let file_ids: BTreeSet<_> = files
        .iter()
        .filter(|file| {
            path.is_empty()
                || file.path == path
                || file
                    .path
                    .strip_prefix(path)
                    .is_some_and(|suffix| suffix.starts_with('/'))
        })
        .map(|file| &file.id)
        .collect();
    entities
        .iter()
        .filter(|entity| file_ids.contains(&entity.file_id))
        .map(|entity| entity.id.clone())
        .collect()
}

fn node(
    name: &str,
    kind: &str,
    scope_id: ScopeId,
    parent_scope_id: Option<ScopeId>,
    entity_ids: Vec<crate::output::v2::EntityId>,
) -> Projection {
    Projection {
        id: v2::stable_id("projection", &[kind, scope_id.as_ref()]),
        name: name.into(),
        scope_id,
        kind: kind.into(),
        parent_scope_id,
        tags: Vec::new(),
        entity_ids,
        relationship_ids: Vec::new(),
    }
}
