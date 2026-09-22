use sysvista_cli::{analyzer::{self, CONTRACT_VERSION}, discovery::Config, output::v2::Diagnostic, scanner};
use std::path::Path;
use std::sync::Mutex;

static PATH_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn embedded_analyzer_handshake_matches_contract() {
    let _guard = PATH_LOCK.lock().unwrap();
    let value = analyzer::handshake().unwrap();
    assert_eq!(value.contract_version, CONTRACT_VERSION);
}

#[test]
fn missing_node_is_an_error_not_a_panic() {
    let _guard = PATH_LOCK.lock().unwrap();
    let old = std::env::var_os("PATH");
    unsafe { std::env::set_var("PATH", "") };
    let result = analyzer::handshake();
    match old { Some(value) => unsafe { std::env::set_var("PATH", value) }, None => unsafe { std::env::remove_var("PATH") } }
    assert!(matches!(result, Err(analyzer::AnalyzerError::Unavailable(_))));
}

#[test]
fn scan_reports_missing_node_and_keeps_heuristic_output() {
    let _guard = PATH_LOCK.lock().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/analyzer_spawn");
    let old = std::env::var_os("PATH");
    unsafe { std::env::set_var("PATH", "") };
    let snapshot = scanner::scan_v2(&root, &Config::default()).unwrap();
    match old { Some(value) => unsafe { std::env::set_var("PATH", value) }, None => unsafe { std::env::remove_var("PATH") } }
    assert!(snapshot.diagnostics.iter().any(|diagnostic| matches!(diagnostic, Diagnostic::AnalyzerUnavailable { severity, .. } if severity == "error")));
}
