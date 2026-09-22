use std::{env, fs, path::PathBuf};

fn main() {
    let source = PathBuf::from("../sysvista-analyzer/dist/analyzer.js");
    println!("cargo:rerun-if-changed={}", source.display());
    let destination = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("analyzer.js");
    if let Ok(bytes) = fs::read(&source) {
        fs::write(destination, bytes).expect("copy analyzer bundle");
    } else {
        // Keep ordinary cargo builds usable before npm setup. CI and release builds
        // build the real bundle first; this fallback reports itself as unavailable.
        fs::write(
            destination,
            b"process.stderr.write('embedded analyzer was not built\\n');process.exit(78);",
        )
        .expect("write analyzer fallback");
    }
}
