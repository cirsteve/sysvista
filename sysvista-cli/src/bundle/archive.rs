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
    pub entry_cap: u64,
    pub archive_cap: u64,
}
impl Default for ArchiveOptions {
    fn default() -> Self {
        Self {
            include_source: true,
            entry_cap: MAX_ENTRY_BYTES,
            archive_cap: MAX_ARCHIVE_BYTES,
        }
    }
}

pub fn write_archive(bundle: &Path, archive: &Path, options: &ArchiveOptions) -> BundleResult<()> {
    let mut manifest: Manifest = serde_json::from_slice(&fs::read(bundle.join("manifest.json"))?)
        .map_err(|e| BundleError::InvalidBundle(e.to_string()))?;
    manifest.source_included = options.include_source;
    let mut entries = Vec::<(String, Vec<u8>)>::new();
    let mut names = vec!["manifest.json".to_string()];
    names.extend(manifest.files.clone());
    names.sort();
    names.dedup();
    for name in names {
        validate_entry_path(Path::new(&name))?;
        let bytes = if name == "manifest.json" {
            let mut bytes = serde_json::to_vec_pretty(&manifest)
                .map_err(|e| BundleError::InvalidBundle(e.to_string()))?;
            bytes.push(b'\n');
            bytes
        } else {
            fs::read(bundle.join(&name))?
        };
        entries.push((name, bytes));
    }
    if options.include_source {
        let index: SourceIndex =
            serde_json::from_slice(&fs::read(bundle.join("source-index.json"))?)
                .map_err(|e| BundleError::InvalidBundle(e.to_string()))?;
        let root = PathBuf::from(&manifest.root);
        let mut hashes = BTreeSet::new();
        for item in index
            .files
            .into_iter()
            .filter(|item| item.source_available && hashes.insert(item.content_hash.clone()))
        {
            validate_entry_path(Path::new(&item.path))?;
            let bytes = fs::read(root.join(&item.path))?;
            if bytes.len() as u64 > options.entry_cap {
                continue;
            }
            let actual = format!("{:x}", Sha256::digest(&bytes));
            if actual != item.content_hash || bytes.len() as u64 != item.byte_length {
                return Err(BundleError::InvalidBundle(format!(
                    "source changed after scan: {}",
                    item.path
                )));
            }
            entries.push((format!("source/{}", item.content_hash), bytes));
        }
    }
    let total: u64 = entries.iter().map(|(_, bytes)| bytes.len() as u64).sum();
    if total > options.archive_cap {
        return Err(BundleError::ArchiveTooLarge {
            bytes: total,
            cap: options.archive_cap,
        });
    }
    if let Some(parent) = archive.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let file = fs::File::create(archive)?;
    write_zip(file, entries)?;
    Ok(())
}

fn write_zip<W: Write + Seek>(writer: W, entries: Vec<(String, Vec<u8>)>) -> BundleResult<()> {
    let mut zip = ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);
    for (name, bytes) in entries {
        validate_entry_path(Path::new(&name))?;
        zip.start_file(name, options)?;
        zip.write_all(&bytes)?;
    }
    zip.finish()?;
    Ok(())
}
