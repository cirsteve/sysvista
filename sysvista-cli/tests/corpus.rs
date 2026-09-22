use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sysvista_cli::discovery::Config;
use sysvista_cli::output::v2::{AnalysisStatus, Diagnostic, Snapshot};
use sysvista_cli::{scanner, validate};

const KINDS: [&str; 12] = [
    "imports",
    "references",
    "calls",
    "contains",
    "depends_on",
    "handles",
    "persists",
    "transforms",
    "consumes",
    "produces",
    "dispatches",
    "invokes_prompt",
];

#[derive(Debug, Deserialize)]
struct Expected {
    relationships: BTreeMap<String, Vec<ExpectedRelationship>>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
struct ExpectedRelationship {
    source: Locator,
    target: Locator,
    #[serde(default)]
    origin: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
struct Locator {
    file: String,
    qualified_name: String,
    declaration_kind: String,
    #[serde(default)]
    discriminator: usize,
}

#[derive(Default)]
struct Counts {
    true_positive: usize,
    actual: usize,
    expected: usize,
}

#[test]
fn labeled_corpus_meets_relationship_floors() {
    let cases = corpus_cases();
    assert_eq!(cases.len(), 13, "the documented corpus must contain all 13 cases");
    let mut aggregate: BTreeMap<&str, Counts> = KINDS
        .into_iter()
        .map(|kind| (kind, Counts::default()))
        .collect();

    for case in cases {
        let expected: Expected = serde_json::from_slice(
            &fs::read(case.join("expected.json")).expect("read expected.json"),
        )
        .expect("parse expected.json");
        assert_eq!(
            expected.relationships.keys().cloned().collect::<BTreeSet<_>>(),
            KINDS.into_iter().map(str::to_owned).collect(),
            "{} must label every relationship kind",
            case.display()
        );

        let config = Config::load(&case).expect("load case config");
        let snapshot = scanner::scan_v2(&case, &config).expect("scan corpus case");
        assert_shared_validation(&case, &snapshot);
        assert_case_specific_behavior(&case, &snapshot);
        let actual = actual_relationships(&snapshot);

        for kind in KINDS {
            let expected_for_kind: BTreeSet<_> = expected.relationships[kind]
                .iter()
                .cloned()
                .collect();
            let actual_for_kind = actual.get(kind).cloned().unwrap_or_default();
            let true_positive = expected_for_kind.intersection(&actual_for_kind).count();
            let precision = ratio(true_positive, actual_for_kind.len());
            let recall = ratio(true_positive, expected_for_kind.len());
            println!(
                "corpus={} kind={kind} precision={precision:.3} recall={recall:.3} tp={true_positive} actual={} expected={}",
                case.file_name().unwrap().to_string_lossy(),
                actual_for_kind.len(),
                expected_for_kind.len()
            );
            let counts = aggregate.get_mut(kind).unwrap();
            counts.true_positive += true_positive;
            counts.actual += actual_for_kind.len();
            counts.expected += expected_for_kind.len();
        }
    }

    for (kind, counts) in aggregate {
        let precision = ratio(counts.true_positive, counts.actual);
        let recall = ratio(counts.true_positive, counts.expected);
        println!(
            "aggregate kind={kind} precision={precision:.3} recall={recall:.3} tp={} actual={} expected={}",
            counts.true_positive, counts.actual, counts.expected
        );
        assert!(precision >= 0.98, "{kind} precision {precision:.3} is below 0.980");
        assert!(recall >= 0.98, "{kind} recall {recall:.3} is below 0.980");
    }
}

fn corpus_cases() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("corpus/cases");
    let mut cases: Vec<_> = fs::read_dir(root)
        .expect("read corpus cases")
        .map(|entry| entry.expect("read case entry").path())
        .filter(|path| path.is_dir())
        .collect();
    cases.sort();
    cases
}

fn assert_shared_validation(case: &Path, snapshot: &Snapshot) {
    let diagnostics = validate::validate(snapshot);
    assert!(
        !diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            Diagnostic::DanglingReference { .. }
                | Diagnostic::Contradiction { .. }
                | Diagnostic::StaleEvidence { .. }
        )),
        "shared validator rejected {}: {diagnostics:#?}",
        case.display()
    );
}

fn assert_case_specific_behavior(case: &Path, snapshot: &Snapshot) {
    match case.file_name().and_then(|name| name.to_str()) {
        Some("parse-error") => assert!(snapshot.diagnostics.iter().any(|diagnostic| {
            matches!(diagnostic, Diagnostic::AnalyzerIssue { severity, span: Some(span), .. }
                if severity == "error" && snapshot.source_files.iter().any(|file| file.id == span.file_id && file.path == "broken.ts"))
        }), "parse-error must diagnose broken.ts"),
        Some("unsupported-language") => assert!(snapshot.source_files.iter().any(|file| {
            file.path == "main.lua" && matches!(file.analysis, AnalysisStatus::None)
        }), "unsupported-language must retain main.lua with analysis=none"),
        Some("dynamic-dispatch") => assert!(snapshot.unresolved_references.iter().any(|item| {
            item.name == "receiver.execute" && item.reason.as_deref() == Some("dynamic or any-typed receiver")
        }), "dynamic dispatch must be preserved as unresolved"),
        Some("missing-dependencies") => assert!(snapshot.unresolved_references.iter().any(|item| {
            item.name == "unavailable" && item.reason.as_deref() == Some("dynamic or any-typed receiver")
        }), "missing dependency must be preserved as unresolved"),
        _ => {}
    }
}

fn actual_relationships(snapshot: &Snapshot) -> BTreeMap<String, BTreeSet<ExpectedRelationship>> {
    let files: HashMap<_, _> = snapshot
        .source_files
        .iter()
        .map(|file| (file.id.clone(), file.path.clone()))
        .collect();
    let mut groups: BTreeMap<(String, String, String), Vec<_>> = BTreeMap::new();
    for entity in &snapshot.entities {
        groups
            .entry((
                files[&entity.file_id].clone(),
                entity.qualified_name.clone(),
                entity.declaration_kind.clone(),
            ))
            .or_default()
            .push(entity);
    }
    let mut locators = HashMap::new();
    for ((file, qualified_name, declaration_kind), mut entities) in groups {
        entities.sort_by_key(|entity| (entity.span.start_line, entity.span.start_column));
        for (discriminator, entity) in entities.into_iter().enumerate() {
            locators.insert(entity.id.clone(), Locator {
                file: file.clone(),
                qualified_name: qualified_name.clone(),
                declaration_kind: declaration_kind.clone(),
                discriminator,
            });
        }
    }

    let mut result: BTreeMap<String, BTreeSet<ExpectedRelationship>> = BTreeMap::new();
    for relationship in &snapshot.relationships {
        let (_, source, target, kind, origin) = relationship.sort_key();
        let (Some(source), Some(target)) = (locators.get(source), locators.get(target)) else {
            panic!("relationship endpoints must resolve: {relationship:?}");
        };
        result.entry(kind.to_owned()).or_default().insert(ExpectedRelationship {
            source: source.clone(),
            target: target.clone(),
            origin: Some(origin.to_owned()),
        });
    }
    result
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 { 1.0 } else { numerator as f64 / denominator as f64 }
}
