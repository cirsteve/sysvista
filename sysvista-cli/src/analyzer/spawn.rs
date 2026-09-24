use std::{
    env, fmt, fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use super::contract::{AnalyzeRequest, AnalyzeResponse, CONTRACT_VERSION, Handshake};

static ANALYZER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/analyzer.js"));

#[derive(Debug)]
pub enum AnalyzerError {
    Unavailable(String),
    ContractMismatch { expected: u32, actual: u32 },
    Protocol(String),
}

impl fmt::Display for AnalyzerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(message) | Self::Protocol(message) => f.write_str(message),
            Self::ContractMismatch { expected, actual } => write!(
                f,
                "analyzer contract mismatch: expected {expected}, got {actual}"
            ),
        }
    }
}

pub fn handshake() -> Result<Handshake, AnalyzerError> {
    let output = Command::new("node")
        .arg(extract()?)
        .arg("--handshake")
        .output()
        .map_err(|error| AnalyzerError::Unavailable(format!("node is unavailable: {error}")))?;
    if !output.status.success() {
        return Err(AnalyzerError::Unavailable(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    let value: Handshake = serde_json::from_slice(&output.stdout)
        .map_err(|error| AnalyzerError::Protocol(error.to_string()))?;
    if value.contract_version != CONTRACT_VERSION {
        return Err(AnalyzerError::ContractMismatch {
            expected: CONTRACT_VERSION,
            actual: value.contract_version,
        });
    }
    Ok(value)
}

pub fn analyze(request: &AnalyzeRequest) -> Result<AnalyzeResponse, AnalyzerError> {
    handshake()?;
    let mut child = Command::new("node")
        .arg(extract()?)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| AnalyzerError::Unavailable(format!("node is unavailable: {error}")))?;
    serde_json::to_writer(child.stdin.as_mut().expect("piped stdin"), request)
        .map_err(|error| AnalyzerError::Protocol(error.to_string()))?;
    child
        .stdin
        .as_mut()
        .expect("piped stdin")
        .flush()
        .map_err(|error| AnalyzerError::Protocol(error.to_string()))?;
    drop(child.stdin.take());
    let output = child
        .wait_with_output()
        .map_err(|error| AnalyzerError::Protocol(error.to_string()))?;
    if !output.status.success() {
        return Err(AnalyzerError::Protocol(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    let response: AnalyzeResponse = serde_json::from_slice(&output.stdout)
        .map_err(|error| AnalyzerError::Protocol(error.to_string()))?;
    if response.contract_version != CONTRACT_VERSION {
        return Err(AnalyzerError::ContractMismatch {
            expected: CONTRACT_VERSION,
            actual: response.contract_version,
        });
    }
    Ok(response)
}

fn extract() -> Result<PathBuf, AnalyzerError> {
    let base = env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir)
        .join("sysvista");
    secure_cache_directory(&base)?;
    let path = base.join(format!("analyzer-{}.js", env!("CARGO_PKG_VERSION")));
    if path.exists() {
        verify_cached_file(&path)?;
    }
    if fs::read(&path).ok().as_deref() != Some(ANALYZER) {
        atomic_install(&base, &path)?;
    }
    verify_cached_file(&path)?;
    Ok(path)
}

#[cfg(unix)]
fn secure_cache_directory(path: &std::path::Path) -> Result<(), AnalyzerError> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    fs::create_dir_all(path).map_err(unavailable)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(unavailable)?;
    let metadata = fs::symlink_metadata(path).map_err(unavailable)?;
    if !metadata.file_type().is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.mode() & 0o077 != 0
    {
        return Err(AnalyzerError::Unavailable(format!(
            "analyzer cache directory is not private and user-owned: {}",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn secure_cache_directory(path: &std::path::Path) -> Result<(), AnalyzerError> {
    fs::create_dir_all(path).map_err(unavailable)
}

#[cfg(unix)]
fn verify_cached_file(path: &std::path::Path) -> Result<(), AnalyzerError> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let metadata = fs::symlink_metadata(path).map_err(unavailable)?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.uid() != unsafe { libc::geteuid() }
    {
        return Err(AnalyzerError::Unavailable(format!(
            "analyzer cache file is not private and user-owned: {}",
            path.display()
        )));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(unavailable)?;
    Ok(())
}

#[cfg(not(unix))]
fn verify_cached_file(path: &std::path::Path) -> Result<(), AnalyzerError> {
    let metadata = fs::symlink_metadata(path).map_err(unavailable)?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(AnalyzerError::Unavailable(format!(
            "analyzer cache file is not regular: {}",
            path.display()
        )));
    }
    Ok(())
}

fn atomic_install(
    directory: &std::path::Path,
    destination: &std::path::Path,
) -> Result<(), AnalyzerError> {
    use std::fs::OpenOptions;
    use std::time::{SystemTime, UNIX_EPOCH};
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = directory.join(format!(".analyzer-{}-{nonce}.tmp", std::process::id()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary).map_err(unavailable)?;
    file.write_all(ANALYZER).map_err(unavailable)?;
    file.sync_all().map_err(unavailable)?;
    drop(file);
    fs::rename(&temporary, destination).map_err(unavailable)
}

fn unavailable(error: std::io::Error) -> AnalyzerError {
    AnalyzerError::Unavailable(error.to_string())
}
