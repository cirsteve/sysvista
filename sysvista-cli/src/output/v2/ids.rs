use crate::discovery::Config;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{path::Path, process::Command};

macro_rules! string_id {
    ($name:ident) => {
        #[derive(
            Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema,
        )]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

string_id!(FileId);
string_id!(EntityId);
string_id!(ModuleId);
string_id!(RelationshipId);
string_id!(ScopeId);

pub fn repository_identity(root: &Path, config: &Config) -> (String, bool) {
    if let Ok(output) = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["remote", "get-url", "origin"])
        .output()
    {
        if output.status.success() {
            let normalized = normalize_remote(String::from_utf8_lossy(&output.stdout).trim());
            if !normalized.is_empty() {
                return (normalized, true);
            }
        }
    }
    if let Some(name) = config
        .repository
        .name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
    {
        return (name.trim().to_owned(), true);
    }
    (
        stable_id("repository-path", &[&root.to_string_lossy()]),
        false,
    )
}

fn normalize_remote(remote: &str) -> String {
    let mut value = remote
        .trim()
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .to_owned();
    if let Some((_, rest)) = value.split_once("://") {
        value = rest.to_owned();
    }
    if value.starts_with("git@") {
        value = value.trim_start_matches("git@").replacen(':', "/", 1);
    }
    if let Some((_, rest)) = value.split_once('@') {
        value = rest.to_owned();
    }
    value.trim_start_matches('/').to_owned()
}

pub fn stable_id(namespace: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    update_framed(&mut hasher, namespace.as_bytes());
    for part in parts {
        update_framed(&mut hasher, part.as_bytes());
    }
    format!("{namespace}:{}", &format!("{:x}", hasher.finalize())[..24])
}

fn update_framed(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}

pub fn file_id(repository: &str, normalized_path: &str) -> FileId {
    FileId(stable_id("file", &[repository, normalized_path]))
}

pub fn scope_id(file_id: &FileId) -> ScopeId {
    ScopeId(stable_id("scope", &[file_id.as_ref()]))
}

pub fn entity_id(
    file_id: &FileId,
    ownership_chain: &str,
    declaration_kind: &str,
    discriminator: usize,
) -> EntityId {
    EntityId(stable_id(
        "entity",
        &[
            file_id.as_ref(),
            ownership_chain,
            declaration_kind,
            &discriminator.to_string(),
        ],
    ))
}

pub fn relationship_id(
    source: &EntityId,
    target: &EntityId,
    kind: &str,
    origin: &str,
) -> RelationshipId {
    RelationshipId(stable_id(
        "relationship",
        &[source.as_ref(), target.as_ref(), kind, origin],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_stable_and_nominal() {
        let file = file_id("example/repo", "src/main.rs");
        assert_eq!(file, file_id("example/repo", "src/main.rs"));
        assert_ne!(file, file_id("example/repo", "src/lib.rs"));
        assert_ne!(
            entity_id(&file, "Thing", "model", 0),
            entity_id(&file, "Thing", "model", 1)
        );
    }

    #[test]
    fn id_parts_use_unambiguous_length_framing() {
        assert_ne!(stable_id("test", &["a\0b"]), stable_id("test", &["a", "b"]));
    }
}
