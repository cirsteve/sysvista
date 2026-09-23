use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use regex::Regex;
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
    for name in ["sysvista.toml", "models.rs", "app.ts", "util.ts", "tsconfig.json"] {
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
    // The fixture must exercise the real Node analyzer, not only the heuristic stage.
    assert!(first.manifest.analyzer_versions.contains_key("typescript"), "{:?}", first.manifest.analyzer_versions);
    assert!(first.relationships.iter().any(|relationship| {
        matches!(relationship, v2::Relationship::Calls { origin, .. } if origin == "resolved")
    }));
    assert!(
        first.diagnostics.iter().any(|diagnostic| matches!(diagnostic, v2::Diagnostic::AnalyzerIssue { message, .. } if message.contains("missing.ts"))),
        "the fixture's missing tsconfig file must produce a path-bearing compiler message"
    );
    let first_output = temp.0.join("first");
    let second_output = temp.0.join("second");
    v2::write_bundle(&first, &first_output).unwrap();
    v2::write_bundle(&second, &second_output).unwrap();

    let mut reordered = first.clone();
    reordered.source_files.reverse();
    reordered.entities.reverse();
    reordered.modules.reverse();
    for module in &mut reordered.modules {
        module.file_ids.reverse();
        module.entity_ids.reverse();
    }
    reordered.relationships.reverse();
    reordered.unresolved_references.reverse();
    reordered.evidence.reverse();
    reordered.claims.reverse();
    for claim in &mut reordered.claims {
        claim.evidence_ids.reverse();
    }
    reordered.payload_contracts.reverse();
    for contract in &mut reordered.payload_contracts {
        contract.producer_ids.reverse();
        contract.consumer_ids.reverse();
    }
    reordered.diagnostics.reverse();
    reordered.projections.reverse();
    for projection in &mut reordered.projections {
        projection.entity_ids.reverse();
        projection.relationship_ids.reverse();
    }
    reordered.findings.reverse();
    reordered.manifest.inventory_entries.reverse();
    let reordered_output = temp.0.join("reordered");
    v2::write_bundle(&reordered, &reordered_output).unwrap();
    for name in [
        "manifest.json",
        "graph.json",
        "diagnostics.json",
        "findings.json",
        "source-index.json",
        "index/scopes.json",
    ] {
        assert_eq!(
            fs::read(first_output.join(name)).unwrap(),
            fs::read(reordered_output.join(name)).unwrap(),
            "{name} changed when input arrays were reordered"
        );
    }

    // The same project under a different absolute root produces identical output
    // apart from the documented volatile manifest fields.
    let moved_root = temp.0.join("elsewhere").join("checkout");
    copy_fixture(&moved_root);
    let moved = scanner::scan_v2(&moved_root, &Config::load(&moved_root).unwrap()).unwrap();
    let moved_output = temp.0.join("moved");
    v2::write_bundle(&moved, &moved_output).unwrap();
    for name in ["graph.json", "diagnostics.json", "findings.json", "source-index.json", "index/scopes.json"] {
        let original = fs::read(first_output.join(name)).unwrap();
        assert_eq!(original, fs::read(moved_output.join(name)).unwrap(), "{name} depends on the checkout location");
        let text = String::from_utf8(original).unwrap();
        for host_path in [root.to_string_lossy(), moved_root.to_string_lossy(), temp.0.to_string_lossy()] {
            assert!(!text.contains(host_path.as_ref()), "{name} leaks {host_path}");
        }
    }

    let first_graph = fs::read(first_output.join("graph.json")).unwrap();
    let second_graph = fs::read(second_output.join("graph.json")).unwrap();
    assert_eq!(first_graph, second_graph);
    let graph_text = String::from_utf8(first_graph.clone()).unwrap();
    assert!(!graph_text.contains(root.to_string_lossy().as_ref()));
    assert!(!graph_text.contains("scanned_at"));
    let iso8601_timestamp = Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}").unwrap();
    assert!(!iso8601_timestamp.is_match(&graph_text));

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
    assert!(scopes.len() >= first.source_files.len());
    assert!(
        scopes
            .windows(2)
            .all(|pair| pair[0]["scope_id"].as_str() <= pair[1]["scope_id"].as_str())
    );
    for scope in scopes {
        let child_scopes = scope["child_scope_ids"].as_array().unwrap();
        assert!(
            child_scopes
                .windows(2)
                .all(|pair| pair[0].as_str() <= pair[1].as_str())
        );
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
