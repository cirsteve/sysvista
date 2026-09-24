use super::Snapshot;
use crate::bundle::BundleResult;
use std::path::Path;

pub fn write_bundle(snapshot: &Snapshot, output: &Path) -> BundleResult<()> {
    crate::bundle::write_directory(snapshot, output)
}
