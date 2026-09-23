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
    bundle::write_archive(
        &directory,
        &archive,
        &ArchiveOptions {
            source_root: Some(root.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    let file = fs::File::open(&archive).unwrap();
    let zip = zip::ZipArchive::new(file).unwrap();
    assert!(zip.file_names().any(|name| name.starts_with("source/")));
    let graph: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("graph.json")).unwrap()).unwrap();
    assert_eq!(
        graph["findings"].as_array().unwrap().len(),
        snapshot.findings.len()
    );
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
    for path in ["..\\escape", "C:/absolute", "./entry", "a//b"] {
        assert!(
            matches!(
                bundle::write_bytes(&temp.0, path, b"x"),
                Err(BundleError::UnsafePath(_))
            ),
            "accepted unsafe portable path {path}"
        );
    }
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

#[test]
fn source_index_represents_files_without_source_metadata() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/findings");
    let mut snapshot = scanner::scan_v2(&root, &Config::load(&root).unwrap()).unwrap();
    let file_id = snapshot.source_files[0].id.clone();
    snapshot.source_files[0].content_hash = None;
    snapshot.source_files[0].byte_length = None;
    let temp = Temp::new();
    bundle::write_directory(&snapshot, &temp.0).unwrap();
    let index: serde_json::Value =
        serde_json::from_slice(&fs::read(temp.0.join("source-index.json")).unwrap()).unwrap();
    let entry = index["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["file_id"] == file_id.0)
        .unwrap();
    assert_eq!(entry["source_available"], false);
    assert!(entry.get("content_hash").is_none());
    assert!(entry.get("byte_length").is_none());
}

#[test]
fn archive_rejects_untrusted_manifest_root() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/findings");
    let snapshot = scanner::scan_v2(&root, &Config::load(&root).unwrap()).unwrap();
    let temp = Temp::new();
    let directory = temp.0.join("bundle");
    bundle::write_directory(&snapshot, &directory).unwrap();
    let untrusted = temp.0.join("untrusted");
    fs::create_dir(&untrusted).unwrap();
    let manifest_path = directory.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["root"] = serde_json::Value::String(untrusted.display().to_string());
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let result = bundle::write_archive(
        &directory,
        &temp.0.join("archive.zip"),
        &ArchiveOptions {
            source_root: Some(root),
            ..Default::default()
        },
    );
    assert!(matches!(result, Err(BundleError::InvalidBundle(_))));
}

#[test]
fn archive_checks_integrity_before_applying_entry_cap() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/findings");
    let snapshot = scanner::scan_v2(&root, &Config::load(&root).unwrap()).unwrap();
    let temp = Temp::new();
    let directory = temp.0.join("bundle");
    bundle::write_directory(&snapshot, &directory).unwrap();
    let index_path = directory.join("source-index.json");
    let mut index: serde_json::Value =
        serde_json::from_slice(&fs::read(&index_path).unwrap()).unwrap();
    index["files"][0]["content_hash"] = serde_json::Value::String("0".repeat(64));
    fs::write(&index_path, serde_json::to_vec_pretty(&index).unwrap()).unwrap();
    let result = bundle::write_archive(
        &directory,
        &temp.0.join("archive.zip"),
        &ArchiveOptions {
            source_root: Some(root),
            entry_cap: 0,
            ..Default::default()
        },
    );
    assert!(matches!(result, Err(BundleError::InvalidBundle(_))));
}

fn findings_bundle(temp: &Temp) -> (PathBuf, PathBuf) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/findings");
    let snapshot = scanner::scan_v2(&root, &Config::load(&root).unwrap()).unwrap();
    let directory = temp.0.join("bundle");
    bundle::write_directory(&snapshot, &directory).unwrap();
    (root, directory)
}

#[test]
fn archive_stops_once_the_running_total_exceeds_its_cap() {
    let temp = Temp::new();
    let (root, directory) = findings_bundle(&temp);
    let result = bundle::write_archive(
        &directory,
        &temp.0.join("archive.zip"),
        &ArchiveOptions { source_root: Some(root), archive_cap: 64, ..Default::default() },
    );
    assert!(matches!(result, Err(BundleError::ArchiveTooLarge { cap: 64, .. })), "{result:?}");
    assert!(!temp.0.join("archive.zip").exists());
}

#[test]
fn sources_over_a_custom_entry_cap_are_marked_unavailable_in_the_archive() {
    let temp = Temp::new();
    let (root, directory) = findings_bundle(&temp);
    let archive = temp.0.join("archive.zip");
    bundle::write_archive(
        &directory,
        &archive,
        &ArchiveOptions { source_root: Some(root), entry_cap: 0, ..Default::default() },
    )
    .unwrap();
    let mut zip = zip::ZipArchive::new(fs::File::open(&archive).unwrap()).unwrap();
    assert!(!zip.file_names().any(|name| name.starts_with("source/")));
    let index: serde_json::Value = serde_json::from_reader(zip.by_name("source-index.json").unwrap()).unwrap();
    let files = index["files"].as_array().unwrap();
    assert!(!files.is_empty());
    assert!(files.iter().all(|file| file["source_available"] == false), "{index:#}");
}
