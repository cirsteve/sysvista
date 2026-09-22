use crate::{
    discovery::Config,
    output::v2::{self, LogicalModule, ModuleId, ScopeId},
};

pub fn derive(repository: &str, config: &Config) -> Vec<LogicalModule> {
    let mut modules: Vec<_> = config
        .modules
        .iter()
        .map(|module| {
            let id = ModuleId(v2::stable_id("module", &[repository, &module.name]));
            LogicalModule {
                scope_id: ScopeId(v2::stable_id("scope", &["logical", id.as_ref()])),
                id,
                name: module.name.clone(),
                selectors: module.selectors.clone(),
                tags: module.tags.clone(),
                file_ids: Vec::new(),
                entity_ids: Vec::new(),
            }
        })
        .collect();
    let id = ModuleId(v2::stable_id("module", &[repository, "Unassigned"]));
    modules.push(LogicalModule {
        scope_id: ScopeId(v2::stable_id("scope", &["logical", id.as_ref()])),
        id,
        name: "Unassigned".into(),
        selectors: Vec::new(),
        tags: Vec::new(),
        file_ids: Vec::new(),
        entity_ids: Vec::new(),
    });
    modules
}
