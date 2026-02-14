use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::output::schema::{ComponentKind, DetectedComponent, SourceLocation};

use super::make_id;

struct ModelPattern {
    regex: Regex,
    name_group: usize,
}

static TS_PATTERNS: LazyLock<Vec<ModelPattern>> = LazyLock::new(|| {
    vec![
        ModelPattern {
            regex: Regex::new(r"(?m)^(?:export\s+)?interface\s+(\w+)").unwrap(),
            name_group: 1,
        },
        ModelPattern {
            regex: Regex::new(r"(?m)^(?:export\s+)?type\s+(\w+)\s*=").unwrap(),
            name_group: 1,
        },
        ModelPattern {
            regex: Regex::new(r"(?m)^(?:export\s+)?enum\s+(\w+)").unwrap(),
            name_group: 1,
        },
    ]
});

static RUST_PATTERNS: LazyLock<Vec<ModelPattern>> = LazyLock::new(|| {
    vec![
        ModelPattern {
            regex: Regex::new(r"(?m)^(?:pub\s+)?struct\s+(\w+)").unwrap(),
            name_group: 1,
        },
        ModelPattern {
            regex: Regex::new(r"(?m)^(?:pub\s+)?enum\s+(\w+)").unwrap(),
            name_group: 1,
        },
    ]
});

static PYTHON_PATTERNS: LazyLock<Vec<ModelPattern>> = LazyLock::new(|| {
    vec![
        ModelPattern {
            regex: Regex::new(r"(?m)^@dataclass\s*\n\s*class\s+(\w+)").unwrap(),
            name_group: 1,
        },
        ModelPattern {
            regex: Regex::new(r"(?m)^class\s+(\w+)\((?:BaseModel|Schema|TypedDict)\)").unwrap(),
            name_group: 1,
        },
    ]
});

static GO_PATTERNS: LazyLock<Vec<ModelPattern>> = LazyLock::new(|| {
    vec![ModelPattern {
        regex: Regex::new(r"(?m)^type\s+(\w+)\s+struct\s*\{").unwrap(),
        name_group: 1,
    }]
});

static PROTO_PATTERNS: LazyLock<Vec<ModelPattern>> = LazyLock::new(|| {
    vec![ModelPattern {
        regex: Regex::new(r"(?m)^message\s+(\w+)\s*\{").unwrap(),
        name_group: 1,
    }]
});

fn extract_ts_fields(content: &str, start: usize) -> Vec<String> {
    let rest = &content[start..];
    let mut fields = Vec::new();
    let mut brace_depth = 0;
    let mut found_open = false;

    for line in rest.lines() {
        let trimmed = line.trim();
        if trimmed.contains('{') {
            found_open = true;
            brace_depth += trimmed.matches('{').count();
            brace_depth -= trimmed.matches('}').count();
            continue;
        }
        if !found_open {
            continue;
        }
        brace_depth += trimmed.matches('{').count();
        brace_depth -= trimmed.matches('}').count();
        if brace_depth == 0 {
            break;
        }
        // Parse field: "name: type" or "name?: type"
        if let Some(colon_pos) = trimmed.find(':') {
            let field_name = trimmed[..colon_pos].trim().trim_end_matches('?');
            if !field_name.is_empty()
                && !field_name.starts_with("//")
                && !field_name.starts_with("/*")
            {
                fields.push(field_name.to_string());
            }
        }
    }

    fields
}

pub fn detect_models(
    content: &str,
    language: &str,
    file: &str,
) -> Vec<DetectedComponent> {
    let patterns: &[ModelPattern] = match language {
        "typescript" | "javascript" => &TS_PATTERNS,
        "rust" => &RUST_PATTERNS,
        "python" => &PYTHON_PATTERNS,
        "go" => &GO_PATTERNS,
        "protobuf" => &PROTO_PATTERNS,
        _ => return Vec::new(),
    };

    let mut components = Vec::new();

    for pattern in patterns {
        for cap in pattern.regex.captures_iter(content) {
            let name = cap[pattern.name_group].to_string();
            let match_start = cap.get(0).unwrap().start();
            let line_num = content[..match_start].lines().count() as u32 + 1;

            let model_fields = if language == "typescript" || language == "javascript" {
                let fields = extract_ts_fields(content, match_start);
                if fields.is_empty() {
                    None
                } else {
                    Some(fields)
                }
            } else {
                None
            };

            components.push(DetectedComponent {
                id: make_id("model", &name, file),
                name,
                kind: ComponentKind::Model,
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
                model_fields,
            });
        }
    }

    components
}
