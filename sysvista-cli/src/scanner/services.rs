use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::output::schema::{ComponentKind, DetectedComponent, SourceLocation};

use super::make_id;

struct ServicePattern {
    regex: Regex,
    name_group: usize,
}

// Decorator-based patterns
static DECORATOR_PATTERNS: LazyLock<Vec<ServicePattern>> = LazyLock::new(|| {
    vec![
        // NestJS / Java Spring
        ServicePattern {
            regex: Regex::new(r"(?m)@(?:Controller|RestController|Injectable|Service)\s*(?:\([^)]*\))?\s*\n\s*(?:export\s+)?class\s+(\w+)").unwrap(),
            name_group: 1,
        },
        // Python Flask/FastAPI
        ServicePattern {
            regex: Regex::new(r"(?m)^class\s+(\w+)\(.*(?:Resource|View|ViewSet|APIView)\)").unwrap(),
            name_group: 1,
        },
    ]
});

// Class patterns in service-like files
static CLASS_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(?:export\s+)?class\s+(\w+)").unwrap());

// Conventional directory names that indicate services
const SERVICE_DIRS: &[&str] = &[
    "services",
    "controllers",
    "handlers",
    "resolvers",
    "middleware",
    "api",
    "crud",
];

fn is_service_dir(file: &str) -> bool {
    let parts: Vec<&str> = file.split('/').collect();
    parts
        .iter()
        .any(|part| SERVICE_DIRS.contains(&part.to_lowercase().as_str()))
}

pub fn detect_services(
    content: &str,
    language: &str,
    file: &str,
) -> Vec<DetectedComponent> {
    let mut components = Vec::new();

    // Check decorator patterns
    for pattern in DECORATOR_PATTERNS.iter() {
        for cap in pattern.regex.captures_iter(content) {
            let name = cap[pattern.name_group].to_string();
            let match_start = cap.get(0).unwrap().start();
            let line_num = content[..match_start].lines().count() as u32 + 1;

            components.push(DetectedComponent {
                id: make_id("service", &name, file),
                name,
                kind: ComponentKind::Service,
                language: language.to_string(),
                source: SourceLocation {
                    file: file.to_string(),
                    line_start: Some(line_num),
                    line_end: None,
                },
                metadata: HashMap::from([("detection".to_string(), "decorator".to_string())]),
                transport_protocol: None,
                http_method: None,
                http_path: None,
                model_fields: None,
                consumes: None,
                produces: None,
            });
        }
    }

    // If no decorator-based detections and this file is in a service-like directory,
    // look for class exports
    if components.is_empty() && is_service_dir(file) {
        for cap in CLASS_PATTERN.captures_iter(content) {
            let name = cap[1].to_string();
            let match_start = cap.get(0).unwrap().start();
            let line_num = content[..match_start].lines().count() as u32 + 1;

            components.push(DetectedComponent {
                id: make_id("service", &name, file),
                name,
                kind: ComponentKind::Service,
                language: language.to_string(),
                source: SourceLocation {
                    file: file.to_string(),
                    line_start: Some(line_num),
                    line_end: None,
                },
                metadata: HashMap::from([(
                    "detection".to_string(),
                    "directory_convention".to_string(),
                )]),
                transport_protocol: None,
                http_method: None,
                http_path: None,
                model_fields: None,
                consumes: None,
                produces: None,
            });
        }
    }

    // For TypeScript/JavaScript: detect exported functions in service directories
    if components.is_empty() && is_service_dir(file) {
        let func_re = LazyLock::force(&EXPORT_FUNC_PATTERN);
        for cap in func_re.captures_iter(content) {
            let name = cap[1].to_string();
            let match_start = cap.get(0).unwrap().start();
            let line_num = content[..match_start].lines().count() as u32 + 1;

            components.push(DetectedComponent {
                id: make_id("service", &name, file),
                name,
                kind: ComponentKind::Service,
                language: language.to_string(),
                source: SourceLocation {
                    file: file.to_string(),
                    line_start: Some(line_num),
                    line_end: None,
                },
                metadata: HashMap::from([(
                    "detection".to_string(),
                    "directory_convention".to_string(),
                )]),
                transport_protocol: None,
                http_method: None,
                http_path: None,
                model_fields: None,
                consumes: None,
                produces: None,
            });
        }
    }

    // For Python: detect bare functions (async def / def) in service directories
    if components.is_empty() && is_service_dir(file) && language == "python" {
        let func_re = LazyLock::force(&PYTHON_FUNC_PATTERN);
        for cap in func_re.captures_iter(content) {
            let name = cap[1].to_string();
            // Filter out dunder methods and private functions
            if name.starts_with("__") || name.starts_with('_') {
                continue;
            }
            let match_start = cap.get(0).unwrap().start();
            let line_num = content[..match_start].lines().count() as u32 + 1;

            components.push(DetectedComponent {
                id: make_id("service", &name, file),
                name,
                kind: ComponentKind::Service,
                language: language.to_string(),
                source: SourceLocation {
                    file: file.to_string(),
                    line_start: Some(line_num),
                    line_end: None,
                },
                metadata: HashMap::from([(
                    "detection".to_string(),
                    "directory_convention".to_string(),
                )]),
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

static EXPORT_FUNC_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^export\s+(?:async\s+)?function\s+(\w+)").unwrap());

static PYTHON_FUNC_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(?:async\s+)?def\s+(\w+)\s*\(").unwrap());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_python_bare_functions_in_crud_dir() {
        let content = r#"
async def create_message(db, data):
    pass

async def get_messages(db, app_id):
    pass

def delete_message(db, msg_id):
    pass
"#;
        let comps = detect_services(content, "python", "src/crud/messages.py");
        let names: Vec<&str> = comps.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["create_message", "get_messages", "delete_message"]);
        assert!(comps.iter().all(|c| c.kind == ComponentKind::Service));
        assert!(comps.iter().all(|c| c.metadata.get("detection").unwrap() == "directory_convention"));
    }

    #[test]
    fn filters_dunder_and_private_functions() {
        let content = r#"
def __init__(self):
    pass

def _helper(x):
    pass

def public_func(x):
    pass
"#;
        let comps = detect_services(content, "python", "src/crud/utils.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "public_func");
    }

    #[test]
    fn no_detection_outside_service_dirs() {
        let content = "async def some_func(x):\n    pass\n";
        let comps = detect_services(content, "python", "src/utils/helpers.py");
        assert!(comps.is_empty());
    }

    #[test]
    fn crud_dir_is_service_dir() {
        assert!(is_service_dir("src/crud/messages.py"));
        assert!(is_service_dir("app/services/user.py"));
        assert!(!is_service_dir("src/models/user.py"));
    }

    #[test]
    fn no_python_detection_for_non_python_language() {
        let content = "def some_func(x):\n    pass\n";
        let comps = detect_services(content, "javascript", "src/crud/helpers.js");
        assert!(comps.is_empty());
    }
}
