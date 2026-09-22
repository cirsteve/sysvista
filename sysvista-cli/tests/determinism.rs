use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;
use sysvista_cli::{discovery::Config, output::v2, scanner};

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("sysvista-{label}-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn copy_fixture(destination: &Path) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/determinism");
    fs::create_dir_all(destination).unwrap();
    for name in ["sysvista.toml", "models.rs"] {
        fs::copy(source.join(name), destination.join(name)).unwrap();
    }
}

fn normalize_spans(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if matches!(key.as_str(), "start_line" | "end_line") {
                    *value = Value::from(0);
                } else {
                    normalize_spans(value);
                }
            }
        }
        Value::Array(values) => values.iter_mut().for_each(normalize_spans),
        _ => {}
    }
}

#[test]
fn graph_is_deterministic_and_line_insensitive() {
    let temp = TempDir::new("determinism");
    let root = temp.0.join("project");
    copy_fixture(&root);
    let config = Config::load(&root).unwrap();

    let first = scanner::scan_v2(&root, &config).unwrap();
    let second = scanner::scan_v2(&root, &config).unwrap();
    let first_output = temp.0.join("first");
    let second_output = temp.0.join("second");
    v2::write_bundle(&first, &first_output).unwrap();
    v2::write_bundle(&second, &second_output).unwrap();

    let first_graph = fs::read(first_output.join("graph.json")).unwrap();
    let second_graph = fs::read(second_output.join("graph.json")).unwrap();
    assert_eq!(first_graph, second_graph);
    let graph_text = String::from_utf8(first_graph.clone()).unwrap();
    assert!(!graph_text.contains(root.to_string_lossy().as_ref()));
    assert!(!graph_text.contains("scanned_at"));
    assert!(!graph_text.contains("T00:00:00Z"));

    let original = fs::read_to_string(root.join("models.rs")).unwrap();
    fs::write(root.join("models.rs"), format!("\n{original}")).unwrap();
    let shifted = scanner::scan_v2(&root, &config).unwrap();
    let shifted_output = temp.0.join("shifted");
    v2::write_bundle(&shifted, &shifted_output).unwrap();

    let mut before: Value = serde_json::from_slice(&first_graph).unwrap();
    let mut after: Value =
        serde_json::from_slice(&fs::read(shifted_output.join("graph.json")).unwrap()).unwrap();
    assert_ne!(before, after, "the source spans should move");
    normalize_spans(&mut before);
    normalize_spans(&mut after);
    assert_eq!(before, after, "only source span line numbers may change");

    let index: Value =
        serde_json::from_slice(&fs::read(first_output.join("index/scopes.json")).unwrap()).unwrap();
    let scopes = index["scopes"].as_array().unwrap();
    assert_eq!(scopes.len(), first.source_files.len());
    assert!(
        scopes
            .windows(2)
            .all(|pair| pair[0]["scope_id"].as_str() <= pair[1]["scope_id"].as_str())
    );
    for scope in scopes {
        let children = scope["child_ids"].as_array().unwrap();
        assert!(
            children
                .windows(2)
                .all(|pair| pair[0].as_str() <= pair[1].as_str())
        );
        let crossing = scope["crossing_relationship_ids"].as_array().unwrap();
        assert!(
            crossing
                .windows(2)
                .all(|pair| pair[0].as_str() <= pair[1].as_str())
        );
    }
}
