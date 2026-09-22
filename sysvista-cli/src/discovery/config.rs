use std::{collections::BTreeMap, fs, io, path::Path};

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
        Ok(())
    }
}
