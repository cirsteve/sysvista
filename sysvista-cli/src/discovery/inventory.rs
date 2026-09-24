use std::{
    fs, io,
    path::{Path, PathBuf},
};

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
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
        let mut ignore_rules = IgnoreRules::new(root)?;
        let mut entries = Vec::new();
        walk(
            root,
            root,
            config,
            includes.as_ref(),
            excludes.as_ref(),
            &mut ignore_rules,
            &mut entries,
        )?;
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(Self { entries })
    }

    pub fn absolute_path(root: &Path, entry: &InventoryEntry) -> PathBuf {
        root.join(&entry.path)
    }
}

struct IgnoreRules {
    global: Gitignore,
    stack: Vec<Gitignore>,
}

impl IgnoreRules {
    fn new(root: &Path) -> io::Result<Self> {
        let (global, error) = GitignoreBuilder::new(root).build_global();
        if let Some(error) = error {
            return Err(io::Error::other(error));
        }
        Ok(Self {
            global,
            stack: vec![build_directory_ignore(root, true)?],
        })
    }

    fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        for matcher in self.stack.iter().rev() {
            let matched = matcher.matched(path, is_dir);
            if !matched.is_none() {
                return matched.is_ignore();
            }
        }
        self.global.matched(path, is_dir).is_ignore()
    }
}

fn build_directory_ignore(directory: &Path, repository_root: bool) -> io::Result<Gitignore> {
    let mut builder = GitignoreBuilder::new(directory);
    for name in [".gitignore", ".ignore"] {
        let path = directory.join(name);
        if path.is_file() {
            if let Some(error) = builder.add(&path) {
                return Err(io::Error::other(error));
            }
        }
    }
    if repository_root {
        if let Some(exclude) = git_exclude_path(directory) {
            if exclude.is_file() {
                if let Some(error) = builder.add(&exclude) {
                    return Err(io::Error::other(error));
                }
            }
        }
    }
    builder.build().map_err(io::Error::other)
}

fn git_exclude_path(root: &Path) -> Option<PathBuf> {
    let dot_git = root.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git.join("info/exclude"));
    }
    let pointer = fs::read_to_string(dot_git).ok()?;
    let git_dir = pointer.trim().strip_prefix("gitdir:")?.trim();
    let git_dir = Path::new(git_dir);
    Some(if git_dir.is_absolute() {
        git_dir.join("info/exclude")
    } else {
        root.join(git_dir).join("info/exclude")
    })
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
    ignore_rules: &mut IgnoreRules,
    entries: &mut Vec<InventoryEntry>,
) -> io::Result<()> {
    let mut children = match fs::read_dir(directory) {
        Ok(children) => {
            let mut collected = Vec::new();
            for child in children {
                match child {
                    Ok(child) => collected.push(child),
                    Err(error) => {
                        let path = relative(root, directory);
                        entries.push(InventoryEntry {
                            path: if path.is_empty() {
                                ".".into()
                            } else {
                                path.clone()
                            },
                            outcome: InventoryOutcome::Failed {
                                diagnostic_id: diagnostic_id("failed", &path, &error.to_string()),
                            },
                        });
                    }
                }
            }
            collected
        }
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
        if child.file_name().to_string_lossy().starts_with('.') {
            entries.push(InventoryEntry {
                path: relative_path,
                outcome: InventoryOutcome::Excluded {
                    rule: "hidden".into(),
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
        if ignore_rules.is_ignored(&path, file_type.is_dir()) {
            entries.push(InventoryEntry {
                path: relative_path,
                outcome: InventoryOutcome::Excluded {
                    rule: "gitignore".into(),
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
            ignore_rules
                .stack
                .push(build_directory_ignore(&path, false)?);
            walk(
                root,
                &path,
                config,
                includes,
                excludes,
                ignore_rules,
                entries,
            )?;
            ignore_rules.stack.pop();
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
            .is_ok_and(|metadata| lacks_read_bits(&metadata))
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

#[cfg(unix)]
fn lacks_read_bits(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o444 == 0
}

#[cfg(not(unix))]
fn lacks_read_bits(_metadata: &fs::Metadata) -> bool {
    false
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
