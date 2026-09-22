use std::{
    fs, io,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use globset::{Glob, GlobSet, GlobSetBuilder};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Config;
use crate::scanner::language;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Inventory {
    pub entries: Vec<InventoryEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InventoryEntry {
    pub path: String,
    pub outcome: InventoryOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InventoryOutcome {
    Included,
    Excluded { rule: String },
    Unsupported,
    Unreadable { io_error: String },
    Failed { diagnostic_id: String },
}

impl Inventory {
    pub fn discover(root: &Path, config: &Config) -> io::Result<Self> {
        let includes = compile(&config.discovery.include)?;
        let excludes = compile(&config.discovery.exclude)?;
        let mut entries = Vec::new();
        walk(
            root,
            root,
            config,
            includes.as_ref(),
            excludes.as_ref(),
            &mut entries,
        )?;
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(Self { entries })
    }

    pub fn absolute_path(root: &Path, entry: &InventoryEntry) -> PathBuf {
        root.join(&entry.path)
    }
}

fn compile(patterns: &[String]) -> io::Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder
            .add(Glob::new(pattern).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?);
    }
    builder
        .build()
        .map(Some)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
}

fn walk(
    root: &Path,
    directory: &Path,
    config: &Config,
    includes: Option<&GlobSet>,
    excludes: Option<&GlobSet>,
    entries: &mut Vec<InventoryEntry>,
) -> io::Result<()> {
    let mut children = match fs::read_dir(directory) {
        Ok(children) => children.filter_map(Result::ok).collect::<Vec<_>>(),
        Err(error) if directory == root => return Err(error),
        Err(error) => {
            let path = relative(root, directory);
            entries.push(InventoryEntry {
                path: path.clone(),
                outcome: InventoryOutcome::Failed {
                    diagnostic_id: diagnostic_id("failed", &path, &error.to_string()),
                },
            });
            return Ok(());
        }
    };
    children.sort_by_key(|entry| entry.file_name());

    for child in children {
        let path = child.path();
        let relative_path = relative(root, &path);
        let file_type = match child.file_type() {
            Ok(value) => value,
            Err(error) => {
                entries.push(InventoryEntry {
                    path: relative_path.clone(),
                    outcome: InventoryOutcome::Failed {
                        diagnostic_id: diagnostic_id("failed", &relative_path, &error.to_string()),
                    },
                });
                continue;
            }
        };

        if file_type.is_symlink() {
            entries.push(InventoryEntry {
                path: relative_path,
                outcome: InventoryOutcome::Excluded {
                    rule: "symlink".into(),
                },
            });
            continue;
        }
        if default_excluded(&relative_path) {
            entries.push(InventoryEntry {
                path: relative_path,
                outcome: InventoryOutcome::Excluded {
                    rule: "default".into(),
                },
            });
            continue;
        }
        if excludes.is_some_and(|set| set.is_match(&relative_path)) {
            let rule = config
                .discovery
                .exclude
                .iter()
                .find(|rule| {
                    Glob::new(rule).is_ok_and(|g| g.compile_matcher().is_match(&relative_path))
                })
                .cloned()
                .unwrap_or_else(|| "exclude".into());
            entries.push(InventoryEntry {
                path: relative_path,
                outcome: InventoryOutcome::Excluded { rule },
            });
            continue;
        }
        if file_type.is_dir() {
            walk(root, &path, config, includes, excludes, entries)?;
            continue;
        }
        if !file_type.is_file() {
            entries.push(InventoryEntry {
                path: relative_path,
                outcome: InventoryOutcome::Unsupported,
            });
            continue;
        }
        if includes.is_some_and(|set| !set.is_match(&relative_path)) {
            entries.push(InventoryEntry {
                path: relative_path,
                outcome: InventoryOutcome::Excluded {
                    rule: "not_included".into(),
                },
            });
            continue;
        }
        if child
            .metadata()
            .is_ok_and(|metadata| metadata.permissions().mode() & 0o444 == 0)
        {
            entries.push(InventoryEntry {
                path: relative_path,
                outcome: InventoryOutcome::Unreadable {
                    io_error: "permission denied".into(),
                },
            });
            continue;
        }
        let supported = language::detect_language_with_config(&path, config).is_some();
        entries.push(InventoryEntry {
            path: relative_path,
            outcome: if supported {
                InventoryOutcome::Included
            } else {
                InventoryOutcome::Unsupported
            },
        });
    }
    Ok(())
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn default_excluded(path: &str) -> bool {
    path.split('/')
        .any(|part| matches!(part, ".git" | "target" | "node_modules"))
}

fn diagnostic_id(kind: &str, path: &str, message: &str) -> String {
    crate::output::v2::stable_id("diagnostic", &[kind, path, message])
}
