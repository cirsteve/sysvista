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
    fs::create_dir_all(&base).map_err(|error| AnalyzerError::Unavailable(error.to_string()))?;
    let path = base.join(format!("analyzer-{}.js", env!("CARGO_PKG_VERSION")));
    if fs::read(&path).ok().as_deref() != Some(ANALYZER) {
        fs::write(&path, ANALYZER)
            .map_err(|error| AnalyzerError::Unavailable(error.to_string()))?;
    }
    Ok(path)
}
