use super::{BundleError, BundleResult};
use std::path::{Component, Path, PathBuf};

pub const MAX_ENTRY_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_ARCHIVE_BYTES: u64 = 512 * 1024 * 1024;

pub fn validate_entry_path(path: &Path) -> BundleResult<PathBuf> {
    if path.is_absolute()
        || path.components().any(|c| {
            matches!(
                c,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
        || path.as_os_str().is_empty()
    {
        return Err(BundleError::UnsafePath(path.into()));
    }
    Ok(path.into())
}
