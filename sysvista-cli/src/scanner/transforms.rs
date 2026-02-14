use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::output::schema::{ComponentKind, DetectedComponent, SourceLocation};

use super::make_id;

// Functions named to_*, from_*, convert*, transform*
static FUNC_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // TypeScript/JavaScript
        Regex::new(r"(?m)^(?:export\s+)?(?:async\s+)?function\s+((?:to|from|convert|transform)\w+)")
            .unwrap(),
        Regex::new(r"(?m)(?:const|let|var)\s+((?:to|from|convert|transform)\w+)\s*=\s*(?:async\s*)?\(")
            .unwrap(),
        // Rust: impl From<A> for B
        Regex::new(r"(?m)^impl\s+From<(\w+)>\s+for\s+(\w+)").unwrap(),
        // Rust: fn to_* / fn from_* / fn convert* / fn transform*
        Regex::new(r"(?m)^(?:pub\s+)?(?:async\s+)?fn\s+((?:to_|from_|convert|transform)\w+)")
            .unwrap(),
        // Python
        Regex::new(r"(?m)^(?:async\s+)?def\s+((?:to_|from_|convert|transform)\w+)").unwrap(),
        // Go
        Regex::new(r"(?m)^func\s+(?:\([^)]+\)\s+)?((?:To|From|Convert|Transform)\w+)").unwrap(),
    ]
});

pub fn detect_transforms(
    content: &str,
    language: &str,
    file: &str,
) -> Vec<DetectedComponent> {
    let mut components = Vec::new();

    for pattern in FUNC_PATTERNS.iter() {
        for cap in pattern.captures_iter(content) {
            // For Rust `impl From<A> for B`, build a special name
            let name = if cap.get(2).is_some() {
                format!("From<{}> for {}", &cap[1], &cap[2])
            } else {
                cap[1].to_string()
            };

            let match_start = cap.get(0).unwrap().start();
            let line_num = content[..match_start].lines().count() as u32 + 1;

            components.push(DetectedComponent {
                id: make_id("transform", &name, file),
                name,
                kind: ComponentKind::Transform,
                language: language.to_string(),
                source: SourceLocation {
                    file: file.to_string(),
                    line_start: Some(line_num),
                    line_end: None,
                },
                metadata: HashMap::new(),
                transport_protocol: None,
                http_method: None,
                http_path: None,
                model_fields: None,
                consumes: None,
                produces: None,
            });
        }
    }

    components
}
