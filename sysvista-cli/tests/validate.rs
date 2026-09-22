use std::path::Path;
use sysvista_cli::{discovery::Config, output::v2::Diagnostic, scanner, validate};

#[test]
fn scanned_snapshot_has_complete_reference_and_membership_coverage() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hierarchy");
    let snapshot = scanner::scan_v2(&root, &Config::load(&root).unwrap()).unwrap();
    let diagnostics = validate::validate(&snapshot);
    assert!(!diagnostics.iter().any(|d| matches!(d, Diagnostic::DanglingReference { .. } | Diagnostic::Coverage { .. })));
}
