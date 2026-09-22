use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use sysvista_cli::{
    bundle::{self, ArchiveOptions, BundleError},
    discovery::Config,
    scanner,
};

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "sysvista-bundle-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn full_bundle_and_archive_have_indexed_sources() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/findings");
    let snapshot = scanner::scan_v2(&root, &Config::load(&root).unwrap()).unwrap();
    let temp = Temp::new();
    let directory = temp.0.join("bundle");
    bundle::write_directory(&snapshot, &directory).unwrap();
    for name in [
        "manifest.json",
        "graph.json",
        "diagnostics.json",
        "findings.json",
        "config.snapshot.toml",
        "source-index.json",
        "index/scopes.json",
    ] {
        assert!(directory.join(name).is_file(), "missing {name}");
    }
    let archive = temp.0.join("with-source.zip");
    bundle::write_archive(&directory, &archive, &ArchiveOptions::default()).unwrap();
    let file = fs::File::open(&archive).unwrap();
    let zip = zip::ZipArchive::new(file).unwrap();
    assert!(zip.file_names().any(|name| name.starts_with("source/")));
    let no_source = temp.0.join("no-source.zip");
    bundle::write_archive(
        &directory,
        &no_source,
        &ArchiveOptions {
            include_source: false,
            ..Default::default()
        },
    )
    .unwrap();
    let file = fs::File::open(no_source).unwrap();
    let zip = zip::ZipArchive::new(file).unwrap();
    assert!(!zip.file_names().any(|name| name.starts_with("source/")));
}

#[test]
fn writer_rejects_traversal_and_absolute_entries() {
    let temp = Temp::new();
    assert!(matches!(
        bundle::write_bytes(&temp.0, "../escape", b"x"),
        Err(BundleError::UnsafePath(_))
    ));
    assert!(matches!(
        bundle::write_bytes(&temp.0, "/absolute", b"x"),
        Err(BundleError::UnsafePath(_))
    ));
}

#[test]
fn oversized_source_is_indexed_as_unavailable_with_diagnostic() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/findings");
    let mut snapshot = scanner::scan_v2(&root, &Config::load(&root).unwrap()).unwrap();
    snapshot.source_files[0].byte_length = Some(bundle::MAX_ENTRY_BYTES + 1);
    let temp = Temp::new();
    bundle::write_directory(&snapshot, &temp.0).unwrap();
    let index: serde_json::Value =
        serde_json::from_slice(&fs::read(temp.0.join("source-index.json")).unwrap()).unwrap();
    assert!(
        index["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["source_available"] == false)
    );
    let diagnostics = fs::read_to_string(temp.0.join("diagnostics.json")).unwrap();
    assert!(diagnostics.contains("source_unavailable"));
}
