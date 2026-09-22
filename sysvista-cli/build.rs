use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let source = PathBuf::from("../sysvista-analyzer/dist/analyzer.js");
    println!("cargo:rerun-if-changed={}", source.display());
    if !source.is_file() {
        let status = Command::new("npm")
            .args(["run", "build"])
            .current_dir("../sysvista-analyzer")
            .status()
            .unwrap_or_else(|error| panic!("the analyzer bundle is required; install Node 22 dependencies and run `make build-analyzer`: {error}"));
        assert!(
            status.success(),
            "the analyzer bundle is required; run `make build-analyzer` before building the CLI"
        );
    }
    let destination = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("analyzer.js");
    fs::copy(&source, destination).expect("copy required analyzer bundle");
}
