use super::{BundleError, BundleResult};
use std::path::PathBuf;

pub const MAX_ENTRY_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_ARCHIVE_BYTES: u64 = 512 * 1024 * 1024;

pub fn validate_entry_path(entry: &str) -> BundleResult<PathBuf> {
    let bytes = entry.as_bytes();
    let has_drive_prefix = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    if entry.is_empty()
        || entry.starts_with('/')
        || entry.contains(['\\', '\0'])
        || has_drive_prefix
        || entry
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."))
    {
        return Err(BundleError::UnsafePath(PathBuf::from(entry)));
    }
    Ok(PathBuf::from(entry))
}
