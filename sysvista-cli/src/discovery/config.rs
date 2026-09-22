use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};

use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub repository: RepositoryConfig,
    pub discovery: DiscoveryConfig,
    // Root-level spellings are accepted for small configurations.
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub extra_extensions: BTreeMap<String, String>,
    pub modules: Vec<ModuleConfig>,
    pub forbidden_dependencies: Vec<ForbiddenDependencyConfig>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ModuleConfig {
    pub name: String,
    #[serde(alias = "globs", alias = "paths")]
    pub selectors: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ForbiddenDependencyConfig {
    #[serde(alias = "source")]
    pub from: String,
    #[serde(alias = "target")]
    pub to: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RepositoryConfig {
    pub name: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DiscoveryConfig {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub extra_extensions: BTreeMap<String, String>,
}

impl Config {
    pub fn load(root: &Path) -> io::Result<Self> {
        let path = root.join("sysvista.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(&path)?;
        let mut config: Self = toml::from_str(&text).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {error}", path.display()),
            )
        })?;
        config.discovery.include.extend(config.include.clone());
        config.discovery.exclude.extend(config.exclude.clone());
        config
            .discovery
            .extra_extensions
            .extend(config.extra_extensions.clone());
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> io::Result<()> {
        for pattern in self.discovery.include.iter().chain(&self.discovery.exclude) {
            globset::Glob::new(pattern).map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("invalid glob {pattern:?}: {error}"),
                )
            })?;
        }
        for extension in self.discovery.extra_extensions.keys() {
            if extension.is_empty() || extension.starts_with('.') || extension.contains('/') {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("invalid extra extension {extension:?}; omit the leading dot"),
                ));
            }
        }
        let mut module_names = BTreeSet::new();
        for module in &self.modules {
            if module.name.trim().is_empty() || module.selectors.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "logical modules require a name and at least one selector",
                ));
            }
            if module.name == "Unassigned" {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "logical module name `Unassigned` is reserved",
                ));
            }
            if !module_names.insert(module.name.as_str()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("duplicate logical module name {:?}", module.name),
                ));
            }
            for selector in &module.selectors {
                globset::Glob::new(selector).map_err(|error| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("invalid module selector {selector:?}: {error}"),
                    )
                })?;
            }
        }
        Ok(())
    }
}
