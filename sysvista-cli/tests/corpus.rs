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
#[serde(deny_unknown_fields)]
struct Expected {
    relationships: BTreeMap<String, Vec<ExpectedRelationship>>,
    /// Case-specific floors by kind and origin, for cases that document a known weakness.
    #[serde(default)]
    floors: BTreeMap<String, BTreeMap<String, Floor>>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(deny_unknown_fields)]
struct ExpectedRelationship {
    source: Locator,
    target: Locator,
    /// Required: an expectation without provenance could be met by a weaker origin.
    origin: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(deny_unknown_fields)]
struct Locator {
    file: String,
    qualified_name: String,
    declaration_kind: String,
    #[serde(default)]
    discriminator: usize,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Floor {
    precision: f64,
    recall: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Floors {
    measured: BTreeMap<String, BTreeMap<String, Floor>>,
    unmeasured: BTreeMap<String, String>,
}

#[derive(Default)]
struct Counts {
    true_positive: usize,
    actual: usize,
    expected: usize,
}

impl Counts {
    /// `None` when nothing was produced: precision is unmeasured, not perfect.
    fn precision(&self) -> Option<f64> {
        ratio(self.true_positive, self.actual)
    }
    /// `None` when nothing was expected: recall is unmeasured, not perfect.
    fn recall(&self) -> Option<f64> {
        ratio(self.true_positive, self.expected)
    }
}

type Key = (&'static str, String);

#[test]
fn labeled_corpus_meets_relationship_floors() {
    let floors = load_floors();
    let cases = corpus_cases();
    assert_eq!(cases.len(), 19, "every documented corpus case must be present");
    let mut aggregate: BTreeMap<Key, Counts> = BTreeMap::new();
    let mut failures = Vec::new();

    for case in &cases {
        let name = case.file_name().unwrap().to_string_lossy().into_owned();
        let expected: Expected = serde_json::from_slice(
            &fs::read(case.join("expected.json")).expect("read expected.json"),
        )
        .unwrap_or_else(|error| panic!("parse {name}/expected.json: {error}"));
        assert_eq!(
            expected.relationships.keys().cloned().collect::<BTreeSet<_>>(),
            KINDS.into_iter().map(str::to_owned).collect(),
            "{name} must label every relationship kind"
        );

        let config = Config::load(case).expect("load case config");
        let snapshot = scanner::scan_v2(case, &config).expect("scan corpus case");
        assert_shared_validation(case, &snapshot);
        assert_case_specific_behavior(case, &snapshot);
        let actual = actual_relationships(&snapshot);

        for kind in KINDS {
            let expected_for_kind: BTreeSet<_> =
                expected.relationships[kind].iter().cloned().collect();
            let actual_for_kind = actual.get(kind).cloned().unwrap_or_default();
            let origins: BTreeSet<_> = expected_for_kind
                .iter()
                .chain(&actual_for_kind)
                .map(|relationship| relationship.origin.clone())
                .collect();
            if let Some(reason) = floors.unmeasured.get(kind) {
                if !origins.is_empty() {
                    failures.push(format!(
                        "{name}: {kind} is documented as unmeasured ({reason}) but the case labels or produces it"
                    ));
                }
                continue;
            }
            for origin in origins {
                let counts = counts(&expected_for_kind, &actual_for_kind, &origin);
                println!(
                    "corpus={name} kind={kind} origin={origin} precision={} recall={} tp={} actual={} expected={}",
                    show(counts.precision()),
                    show(counts.recall()),
                    counts.true_positive,
                    counts.actual,
                    counts.expected
                );
                let floor = expected
                    .floors
                    .get(kind)
                    .and_then(|by_origin| by_origin.get(&origin))
                    .or_else(|| floors.measured.get(kind).and_then(|by_origin| by_origin.get(&origin)));
                match floor {
                    Some(floor) => check(&mut failures, &format!("{name} {kind}/{origin}"), &counts, floor),
                    None => failures.push(format!(
                        "{name}: {kind}/{origin} has no floor in corpus/floors.json; measure it and add one"
                    )),
                }
                let total = aggregate.entry((kind, origin)).or_default();
                total.true_positive += counts.true_positive;
                total.actual += counts.actual;
                total.expected += counts.expected;
            }
        }
    }

    for (kind, by_origin) in &floors.measured {
        for (origin, floor) in by_origin {
            let counts = aggregate
                .remove(&(KINDS.into_iter().find(|k| k == kind).expect("known kind"), origin.clone()))
                .unwrap_or_default();
            println!(
                "aggregate kind={kind} origin={origin} precision={} recall={} tp={} actual={} expected={}",
                show(counts.precision()),
                show(counts.recall()),
                counts.true_positive,
                counts.actual,
                counts.expected
            );
            if counts.expected == 0 {
                failures.push(format!(
                    "{kind}/{origin} has a floor but no case expects it; a floor needs support"
                ));
            }
            check(&mut failures, &format!("aggregate {kind}/{origin}"), &counts, floor);
        }
    }
    assert!(failures.is_empty(), "corpus floors failed:\n{}", failures.join("\n"));
}

#[test]
fn floors_partition_every_relationship_kind() {
    let floors = load_floors();
    let measured: BTreeSet<_> = floors.measured.keys().map(String::as_str).collect();
    let unmeasured: BTreeSet<_> = floors.unmeasured.keys().map(String::as_str).collect();
    assert!(measured.is_disjoint(&unmeasured), "a kind cannot be both measured and unmeasured");
    assert_eq!(
        measured.union(&unmeasured).copied().collect::<BTreeSet<_>>(),
        KINDS.into_iter().collect(),
        "floors.json must list every kind as measured or unmeasured"
    );
    assert!(floors.unmeasured.values().all(|reason| !reason.trim().is_empty()));
}

#[test]
fn expectations_without_an_origin_are_rejected() {
    let missing = r#"{"source":{"file":"a.ts","qualified_name":"a","declaration_kind":"function"},
        "target":{"file":"b.ts","qualified_name":"b","declaration_kind":"function"}}"#;
    assert!(serde_json::from_str::<ExpectedRelationship>(missing).is_err());
}

#[test]
fn a_dropped_expectation_or_downgraded_origin_fails_its_floor() {
    let locator = |name: &str| Locator {
        file: "main.ts".into(),
        qualified_name: name.into(),
        declaration_kind: "function".into(),
        discriminator: 0,
    };
    let relationship = |origin: &str| ExpectedRelationship {
        source: locator("run"),
        target: locator("work"),
        origin: origin.into(),
    };
    let expected = BTreeSet::from([relationship("resolved")]);
    let strict = Floor { precision: 0.98, recall: 0.98 };
    let mut failures = Vec::new();
    // The analyzer stopped producing the edge.
    check(&mut failures, "dropped", &counts(&expected, &BTreeSet::new(), "resolved"), &strict);
    // Only a heuristic edge remains for a resolved expectation.
    let downgraded = BTreeSet::from([relationship("heuristic")]);
    check(&mut failures, "downgraded", &counts(&expected, &downgraded, "resolved"), &strict);
    assert_eq!(failures.len(), 2, "{failures:?}");
}

fn load_floors() -> Floors {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("corpus/floors.json");
    serde_json::from_slice(&fs::read(&path).expect("read corpus/floors.json")).expect("parse corpus/floors.json")
}

fn counts(
    expected: &BTreeSet<ExpectedRelationship>,
    actual: &BTreeSet<ExpectedRelationship>,
    origin: &str,
) -> Counts {
    let expected: BTreeSet<_> = expected.iter().filter(|r| r.origin == origin).collect();
    let actual: BTreeSet<_> = actual.iter().filter(|r| r.origin == origin).collect();
    Counts {
        true_positive: expected.intersection(&actual).count(),
        actual: actual.len(),
        expected: expected.len(),
    }
}

fn check(failures: &mut Vec<String>, label: &str, counts: &Counts, floor: &Floor) {
    if let Some(precision) = counts.precision().filter(|value| *value < floor.precision) {
        failures.push(format!("{label}: precision {precision:.3} is below {:.3}", floor.precision));
    }
    if let Some(recall) = counts.recall().filter(|value| *value < floor.recall) {
        failures.push(format!("{label}: recall {recall:.3} is below {:.3}", floor.recall));
    }
}

fn show(value: Option<f64>) -> String {
    value.map_or_else(|| "unmeasured".into(), |value| format!("{value:.3}"))
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
            Diagnostic::DanglingReference { .. } | Diagnostic::StaleEvidence { .. }
        )),
        "shared validator rejected {}: {diagnostics:#?}",
        case.display()
    );
}

fn entity<'a>(snapshot: &'a Snapshot, path: &str, qualified_name: &str, kind: &str) -> Vec<&'a sysvista_cli::output::v2::CodeEntity> {
    let file = snapshot.source_files.iter().find(|file| file.path == path).map(|file| &file.id);
    snapshot
        .entities
        .iter()
        .filter(|entity| Some(&entity.file_id) == file && entity.qualified_name == qualified_name && entity.declaration_kind == kind)
        .collect()
}

fn analyzer_issues(snapshot: &Snapshot) -> Vec<&str> {
    snapshot
        .diagnostics
        .iter()
        .filter_map(|diagnostic| match diagnostic {
            Diagnostic::AnalyzerIssue { message, .. } => Some(message.as_str()),
            _ => None,
        })
        .collect()
}

fn assert_case_specific_behavior(case: &Path, snapshot: &Snapshot) {
    match case.file_name().and_then(|name| name.to_str()) {
        Some("parse-error") => assert!(snapshot.diagnostics.iter().any(|diagnostic| {
            matches!(diagnostic, Diagnostic::AnalyzerIssue { severity, span: Some(span), .. }
                if severity == "error" && snapshot.source_files.iter().any(|file| file.id == span.file_id && file.path == "broken.ts"))
        }), "parse-error must diagnose broken.ts"),
        Some("unsupported-language") => assert!(snapshot.source_files.iter().any(|file| {
            file.path == "main.lua" && matches!(file.analysis, AnalysisStatus::Unsupported)
        }), "unsupported-language must retain main.lua as unsupported"),
        Some("dynamic-dispatch") => assert!(snapshot.unresolved_references.iter().any(|item| {
            item.name == "receiver.execute" && item.reason.as_deref() == Some("dynamic or any-typed receiver")
        }), "dynamic dispatch must be preserved as unresolved"),
        Some("missing-dependencies") => assert!(snapshot.unresolved_references.iter().any(|item| {
            item.name == "unavailable" && item.reason.as_deref() == Some("dynamic or any-typed receiver")
        }), "missing dependency must be preserved as unresolved"),
        Some("solution-tsconfig") => {
            // Referenced projects supply real options, so default-library calls resolve as external.
            assert!(snapshot.unresolved_references.is_empty(), "{:#?}", snapshot.unresolved_references);
            assert!(analyzer_issues(snapshot).is_empty(), "{:#?}", analyzer_issues(snapshot));
        }
        Some("nested-ownership") => {
            let implementation = entity(snapshot, "math.ts", "parse", "function")
                .into_iter()
                .max_by_key(|entity| entity.span.start_line)
                .expect("parse implementation");
            let normalize = entity(snapshot, "math.ts", "parse.normalize", "variable");
            assert_eq!(normalize.len(), 1);
            assert_eq!(normalize[0].owner_id.as_ref(), Some(&implementation.id), "nested declarations belong to the implementation");
            for (name, kind) in [("Widget.total", "getter"), ("Widget.total", "setter"), ("Widget.onClick", "property"), ("Widget.constructor", "constructor")] {
                assert_eq!(entity(snapshot, "widget.ts", name, kind).len(), 1, "{name} {kind}");
            }
        }
        Some("overlapping-projects") => {
            assert_eq!(entity(snapshot, "src/core.ts", "core", "function").len(), 1);
            assert!(
                analyzer_issues(snapshot).iter().any(|message| message.contains("src/core.ts is included by 2 tsconfig projects")),
                "overlap must be flagged: {:#?}",
                analyzer_issues(snapshot)
            );
        }
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
            locators.insert(
                entity.id.clone(),
                Locator {
                    file: file.clone(),
                    qualified_name: qualified_name.clone(),
                    declaration_kind: declaration_kind.clone(),
                    discriminator,
                },
            );
        }
    }

    let mut result: BTreeMap<String, BTreeSet<ExpectedRelationship>> = BTreeMap::new();
    for relationship in &snapshot.relationships {
        let (_, source, target, kind, origin) = relationship.sort_key();
        let (Some(source), Some(target)) = (locators.get(source), locators.get(target)) else {
            panic!("relationship endpoints must resolve: {relationship:?}");
        };
        result
            .entry(kind.to_owned())
            .or_default()
            .insert(ExpectedRelationship {
                source: source.clone(),
                target: target.clone(),
                origin: origin.to_owned(),
            });
    }
    result
}

fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 / denominator as f64)
}
