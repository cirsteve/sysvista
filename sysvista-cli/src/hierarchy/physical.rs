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
        Vec::new(),
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
    for dir in &dirs {
        let scope = ScopeId(v2::stable_id("scope", &["directory", repository, dir]));
        let parent_path = Path::new(dir)
            .parent()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .filter(|p| !p.is_empty());
        let parent = parent_path
            .map(|p| ScopeId(v2::stable_id("scope", &["directory", repository, &p])))
            .unwrap_or_else(|| root.clone());
        out.push(node(dir, "directory", scope, Some(parent), Vec::new()));
    }
    for marker in ["package.json", "Cargo.toml", "pyproject.toml"] {
        for entry in inventory
            .entries
            .iter()
            .filter(|e| e.path == marker || e.path.ends_with(&format!("/{marker}")))
        {
            let package_path = Path::new(&entry.path)
                .parent()
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
            let scope = ScopeId(v2::stable_id(
                "scope",
                &["package", repository, &package_path],
            ));
            let parent = if package_path.is_empty() {
                root.clone()
            } else {
                ScopeId(v2::stable_id(
                    "scope",
                    &["directory", repository, &package_path],
                ))
            };
            out.push(node(
                if package_path.is_empty() {
                    repository
                } else {
                    &package_path
                },
                "package",
                scope,
                Some(parent),
                Vec::new(),
            ));
        }
    }
    for (path, file) in file_by_path {
        let parent_path = Path::new(path)
            .parent()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .filter(|p| !p.is_empty());
        let parent = parent_path
            .map(|p| ScopeId(v2::stable_id("scope", &["directory", repository, &p])))
            .unwrap_or_else(|| root.clone());
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
    for entity in entities {
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
