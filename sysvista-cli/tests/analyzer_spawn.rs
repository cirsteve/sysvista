use std::path::Path;
use std::sync::Mutex;
#[cfg(unix)]
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    time::{SystemTime, UNIX_EPOCH},
};
use sysvista_cli::{
    analyzer::{self, CONTRACT_VERSION},
    discovery::Config,
    output::v2::{AnalysisStatus, Diagnostic},
    scanner,
};

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
    match old {
        Some(value) => unsafe { std::env::set_var("PATH", value) },
        None => unsafe { std::env::remove_var("PATH") },
    }
    assert!(matches!(
        result,
        Err(analyzer::AnalyzerError::Unavailable(_))
    ));
}

#[test]
fn scan_reports_missing_node_and_keeps_heuristic_output() {
    let _guard = PATH_LOCK.lock().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/analyzer_spawn");
    let old = std::env::var_os("PATH");
    unsafe { std::env::set_var("PATH", "") };
    let snapshot = scanner::scan_v2(&root, &Config::default()).unwrap();
    match old {
        Some(value) => unsafe { std::env::set_var("PATH", value) },
        None => unsafe { std::env::remove_var("PATH") },
    }
    assert!(snapshot.diagnostics.iter().any(|diagnostic| matches!(diagnostic, Diagnostic::AnalyzerUnavailable { severity, .. } if severity == "error")));
    assert!(
        snapshot
            .source_files
            .iter()
            .filter(|file| file.language.as_deref() == Some("typescript"))
            .all(|file| matches!(file.analysis, AnalysisStatus::Failed { .. }))
    );
}

#[cfg(unix)]
#[test]
fn extracted_bundle_uses_private_permissions() {
    let _guard = PATH_LOCK.lock().unwrap();
    let root = std::env::temp_dir().join(format!(
        "sysvista-cache-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let old = std::env::var_os("XDG_CACHE_HOME");
    unsafe { std::env::set_var("XDG_CACHE_HOME", &root) };
    analyzer::handshake().unwrap();
    match old {
        Some(value) => unsafe { std::env::set_var("XDG_CACHE_HOME", value) },
        None => unsafe { std::env::remove_var("XDG_CACHE_HOME") },
    }
    let directory = root.join("sysvista");
    let bundle = directory.join(format!("analyzer-{}.js", env!("CARGO_PKG_VERSION")));
    assert_eq!(
        fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(&bundle).unwrap().permissions().mode() & 0o777,
        0o600
    );
    fs::remove_dir_all(root).unwrap();
}
