use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

const ANALYZER: &str = "../sysvista-analyzer";

fn analyzer_cache_root() -> PathBuf {
    env::var_os("XDG_CACHE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or_else(env::temp_dir)
        .join("sysvista")
}

fn main() {
    // Rebuild the embedded analyzer whenever anything that feeds its bundle changes.
    // The bundle itself is not watched: this script writes it, and watching an output
    // makes cargo rerun the script on every build.
    for input in [
        "src",
        "scripts",
        "package.json",
        "package-lock.json",
        "tsconfig.json",
    ] {
        println!("cargo:rerun-if-changed={ANALYZER}/{input}");
    }
    println!("cargo:rerun-if-env-changed=XDG_CACHE_HOME");
    println!("cargo:rerun-if-env-changed=HOME");
    let analyzer = Path::new(ANALYZER);
    let bundle = analyzer.join("dist/analyzer.js");
    if analyzer.join("node_modules").is_dir() {
        let status = Command::new("npm")
            .args(["run", "build"])
            .env("npm_config_cache", analyzer_cache_root().join("npm"))
            .current_dir(analyzer)
            .status()
            .unwrap_or_else(|error| {
                panic!("could not run `npm run build` for the analyzer: {error}")
            });
        assert!(
            status.success(),
            "building the analyzer bundle failed; see the npm output above"
        );
    } else if bundle.is_file() {
        println!(
            "cargo:warning=sysvista-analyzer dependencies are not installed; embedding the existing dist/analyzer.js, which may be stale. Run `make build-analyzer` to rebuild it."
        );
    } else {
        panic!(
            "the analyzer bundle is required; install Node 22 dependencies and run `make build-analyzer`"
        );
    }
    let destination = Path::new(&env::var_os("OUT_DIR").expect("OUT_DIR")).join("analyzer.js");
    fs::copy(&bundle, destination).expect("copy required analyzer bundle");
}
