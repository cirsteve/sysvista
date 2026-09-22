use std::path::Path;

use crate::discovery::Config;

pub fn detect_language(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "ts" | "tsx" => Some("typescript"),
        "js" | "jsx" | "mjs" | "cjs" => Some("javascript"),
        "rs" => Some("rust"),
        "py" => Some("python"),
        "go" => Some("go"),
        "java" => Some("java"),
        "kt" | "kts" => Some("kotlin"),
        "cs" => Some("csharp"),
        "rb" => Some("ruby"),
        "proto" => Some("protobuf"),
        "graphql" | "gql" => Some("graphql"),
        _ => None,
    }
}

pub fn detect_language_with_config<'a>(path: &Path, config: &'a Config) -> Option<&'a str> {
    let ext = path.extension()?.to_str()?;
    if let Some(language) = detect_language(path) {
        return Some(language);
    }
    config
        .discovery
        .extra_extensions
        .get(ext)
        .map(String::as_str)
}
