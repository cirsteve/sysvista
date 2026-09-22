use crate::output::v2::{self, CodeEntity, Diagnostic, LogicalModule, SourceFile};
use globset::Glob;
use std::collections::BTreeMap;

pub fn assign(
    modules: &mut [LogicalModule],
    files: &[SourceFile],
    entities: &[CodeEntity],
) -> Vec<Diagnostic> {
    let unassigned = modules
        .iter()
        .position(|m| m.name == "Unassigned")
        .expect("Unassigned module");
    let mut diagnostics = Vec::new();
    let mut owners = BTreeMap::new();
    for file in files {
        let mut candidates = Vec::new();
        for (index, module) in modules
            .iter()
            .enumerate()
            .filter(|(_, m)| m.name != "Unassigned")
        {
            for selector in &module.selectors {
                if Glob::new(selector).is_ok_and(|g| g.compile_matcher().is_match(&file.path)) {
                    candidates.push((literal_prefix(selector), index, module.name.clone()));
                }
            }
        }
        candidates.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        let owner = if candidates.is_empty() {
            unassigned
        } else {
            let best = candidates[0].0;
            let mut tied: Vec<_> = candidates.iter().filter(|c| c.0 == best).collect();
            tied.sort_by_key(|c| c.1);
            tied.dedup_by_key(|c| c.1);
            if tied.len() > 1 {
                let names: Vec<_> = tied.iter().map(|c| c.2.clone()).collect();
                diagnostics.push(Diagnostic::MembershipConflict {
                    id: v2::stable_id(
                        "diagnostic",
                        &["membership_conflict", file.id.as_ref(), &names.join("|")],
                    ),
                    file_id: file.id.clone(),
                    path: file.path.clone(),
                    modules: names.clone(),
                    message: format!(
                        "{} matches equally specific modules: {}",
                        file.path,
                        names.join(", ")
                    ),
                });
                unassigned
            } else {
                candidates[0].1
            }
        };
        modules[owner].file_ids.push(file.id.clone());
        owners.insert(file.id.clone(), owner);
    }
    for entity in entities {
        if let Some(owner) = owners.get(&entity.file_id) {
            modules[*owner].entity_ids.push(entity.id.clone());
        }
    }
    for module in modules {
        module.file_ids.sort();
        module.file_ids.dedup();
        module.entity_ids.sort();
        module.entity_ids.dedup();
    }
    diagnostics
}

fn literal_prefix(pattern: &str) -> usize {
    pattern
        .chars()
        .take_while(|c| !matches!(c, '*' | '?' | '[' | '{' | '!'))
        .map(char::len_utf8)
        .sum()
}
