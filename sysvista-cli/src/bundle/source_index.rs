use crate::output::v2::{self, Diagnostic, FileId, Snapshot};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SourceIndex {
    pub files: Vec<SourceIndexEntry>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceIndexEntry {
    pub file_id: FileId,
    pub path: String,
    pub content_hash: String,
    pub byte_length: u64,
    pub source_available: bool,
}

impl SourceIndex {
    pub fn from_snapshot(snapshot: &Snapshot, cap: u64, diagnostics: &mut Vec<Diagnostic>) -> Self {
        let mut files = Vec::new();
        for file in &snapshot.source_files {
            let (Some(hash), Some(bytes)) = (file.content_hash.clone(), file.byte_length) else {
                continue;
            };
            let available = bytes <= cap;
            if !available {
                diagnostics.push(Diagnostic::SourceUnavailable {
                    id: v2::stable_id(
                        "diagnostic",
                        &["source_unavailable", file.id.as_ref(), &bytes.to_string()],
                    ),
                    file_id: file.id.clone(),
                    path: file.path.clone(),
                    message: format!("source exceeds {cap} byte cap"),
                });
            }
            files.push(SourceIndexEntry {
                file_id: file.id.clone(),
                path: file.path.clone(),
                content_hash: hash,
                byte_length: bytes,
                source_available: available,
            });
        }
        files.sort_by(|a, b| a.file_id.cmp(&b.file_id));
        Self { files }
    }
}
