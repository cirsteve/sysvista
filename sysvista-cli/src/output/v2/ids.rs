use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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

pub fn stable_id(namespace: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(namespace.as_bytes());
    for part in parts {
        hasher.update([0]);
        hasher.update(part.as_bytes());
    }
    format!("{namespace}:{}", &format!("{:x}", hasher.finalize())[..24])
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
}
