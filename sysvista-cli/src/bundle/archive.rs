use super::{
    BundleError, BundleResult, MAX_ARCHIVE_BYTES, MAX_ENTRY_BYTES, SourceIndex, validate_entry_path,
};
use crate::output::v2::Manifest;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    io::{Seek, Write},
    path::{Path, PathBuf},
};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

#[derive(Clone, Debug)]
pub struct ArchiveOptions {
    pub include_source: bool,
    /// Explicitly trusted repository root used to resolve source-index paths.
    pub source_root: Option<PathBuf>,
    pub entry_cap: u64,
    pub archive_cap: u64,
}
impl Default for ArchiveOptions {
    fn default() -> Self {
        Self {
            include_source: true,
            source_root: None,
            entry_cap: MAX_ENTRY_BYTES,
            archive_cap: MAX_ARCHIVE_BYTES,
        }
    }
}

pub fn write_archive(bundle: &Path, archive: &Path, options: &ArchiveOptions) -> BundleResult<()> {
    let mut manifest: Manifest = serde_json::from_slice(&fs::read(bundle.join("manifest.json"))?)
        .map_err(|e| BundleError::InvalidBundle(e.to_string()))?;
    manifest.source_included = options.include_source;
    let mut entries = Entries::new(options.archive_cap);
    let mut index = None;
    if options.include_source {
        let mut source_index: SourceIndex =
            serde_json::from_slice(&fs::read(bundle.join("source-index.json"))?)
                .map_err(|e| BundleError::InvalidBundle(e.to_string()))?;
        add_sources(&mut entries, &mut source_index, &manifest, options)?;
        index = Some(source_index);
    }
    let mut names = vec!["manifest.json".to_string()];
    names.extend(manifest.files.clone());
    names.sort();
    names.dedup();
    for name in names {
        validate_entry_path(&name)?;
        let bytes = match (name.as_str(), &index) {
            ("manifest.json", _) => pretty(&manifest)?,
            // The archive's index reflects which sources the archive actually holds.
            ("source-index.json", Some(index)) => pretty(index)?,
            _ => {
                entries.reserve(fs::metadata(bundle.join(&name))?.len())?;
                fs::read(bundle.join(&name))?
            }
        };
        entries.push(name, bytes)?;
    }
    if let Some(parent) = archive.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let file = fs::File::create(archive)?;
    write_zip(file, entries.items)?;
    Ok(())
}

/// Copy each available source into the archive, verifying it has not changed since the
/// scan. A source over the entry cap is left out and marked unavailable in the index.
fn add_sources(
    entries: &mut Entries,
    index: &mut SourceIndex,
    manifest: &Manifest,
    options: &ArchiveOptions,
) -> BundleResult<()> {
    let configured_root = options.source_root.as_ref().ok_or_else(|| {
        BundleError::InvalidBundle("including source requires an explicit trusted source root".into())
    })?;
    let root = configured_root.canonicalize()?;
    let manifest_root = PathBuf::from(&manifest.root).canonicalize()?;
    if root != manifest_root {
        return Err(BundleError::InvalidBundle(format!(
            "trusted source root {} does not match scanned root {}",
            root.display(),
            manifest_root.display()
        )));
    }
    let mut archived = BTreeSet::<String>::new();
    let mut skipped = BTreeSet::<String>::new();
    for item in index.files.iter().filter(|item| item.source_available) {
        let (Some(content_hash), Some(byte_length)) = (item.content_hash.as_ref(), item.byte_length) else {
            return Err(BundleError::InvalidBundle(format!(
                "available source lacks hash or byte length: {}",
                item.path
            )));
        };
        if archived.contains(content_hash) || skipped.contains(content_hash) {
            continue;
        }
        let relative = validate_entry_path(&item.path)?;
        let source = root.join(relative).canonicalize()?;
        if !source.starts_with(&root) {
            return Err(BundleError::UnsafePath(PathBuf::from(&item.path)));
        }
        let fits = byte_length <= options.entry_cap;
        if fits {
            entries.reserve(byte_length)?;
        }
        let bytes = fs::read(source)?;
        let actual = format!("{:x}", Sha256::digest(&bytes));
        if actual != *content_hash || bytes.len() as u64 != byte_length {
            return Err(BundleError::InvalidBundle(format!("source changed after scan: {}", item.path)));
        }
        if fits {
            entries.push(format!("source/{content_hash}"), bytes)?;
            archived.insert(content_hash.clone());
        } else {
            skipped.insert(content_hash.clone());
        }
    }
    for item in &mut index.files {
        if item.content_hash.as_ref().is_some_and(|hash| skipped.contains(hash)) {
            item.source_available = false;
        }
    }
    Ok(())
}

/// Archive entries with a running size total, so an oversized bundle is rejected
/// before everything is held in memory.
struct Entries {
    items: Vec<(String, Vec<u8>)>,
    total: u64,
    cap: u64,
}

impl Entries {
    fn new(cap: u64) -> Self {
        Self { items: Vec::new(), total: 0, cap }
    }

    /// Fail before reading `bytes` more when they would exceed the cap.
    fn reserve(&self, bytes: u64) -> BundleResult<()> {
        let total = self.total.saturating_add(bytes);
        if total > self.cap {
            return Err(BundleError::ArchiveTooLarge { bytes: total, cap: self.cap });
        }
        Ok(())
    }

    fn push(&mut self, name: String, bytes: Vec<u8>) -> BundleResult<()> {
        self.reserve(bytes.len() as u64)?;
        self.total += bytes.len() as u64;
        self.items.push((name, bytes));
        Ok(())
    }
}

fn pretty<T: serde::Serialize>(value: &T) -> BundleResult<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| BundleError::InvalidBundle(e.to_string()))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn write_zip<W: Write + Seek>(writer: W, entries: Vec<(String, Vec<u8>)>) -> BundleResult<()> {
    let mut zip = ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);
    for (name, bytes) in entries {
        validate_entry_path(&name)?;
        zip.start_file(name, options)?;
        zip.write_all(&bytes)?;
    }
    zip.finish()?;
    Ok(())
}
