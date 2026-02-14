use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::output::schema::{
    ComponentKind, DetectedComponent, SourceLocation, TransportProtocol,
};

use super::make_id;

struct RoutePattern {
    regex: Regex,
    method_group: usize,
    path_group: usize,
    protocol: TransportProtocol,
}

static HTTP_PATTERNS: LazyLock<Vec<RoutePattern>> = LazyLock::new(|| {
    vec![
        // Express-style: router.get("/path", ...) or app.post("/path", ...)
        RoutePattern {
            regex: Regex::new(
                r#"(?m)(?:router|app|server)\.(get|post|put|patch|delete|all)\s*\(\s*['"]([^'"]+)['"]"#,
            )
            .unwrap(),
            method_group: 1,
            path_group: 2,
            protocol: TransportProtocol::Http,
        },
        // NestJS decorators: @Get("/path"), @Post("/path")
        RoutePattern {
            regex: Regex::new(
                r#"(?m)@(Get|Post|Put|Patch|Delete)\s*\(\s*['"]([^'"]+)['"]"#,
            )
            .unwrap(),
            method_group: 1,
            path_group: 2,
            protocol: TransportProtocol::Http,
        },
        // Python Flask/FastAPI: @app.get("/path") or @router.post("/path")
        RoutePattern {
            regex: Regex::new(
                r#"(?m)@(?:app|router|api)\.(get|post|put|patch|delete)\s*\(\s*['"]([^'"]+)['"]"#,
            )
            .unwrap(),
            method_group: 1,
            path_group: 2,
            protocol: TransportProtocol::Http,
        },
        // Java Spring: @GetMapping("/path"), @PostMapping("/path")
        RoutePattern {
            regex: Regex::new(
                r#"(?m)@(Get|Post|Put|Patch|Delete)Mapping\s*\(\s*(?:value\s*=\s*)?['"]([^'"]+)['"]"#,
            )
            .unwrap(),
            method_group: 1,
            path_group: 2,
            protocol: TransportProtocol::Http,
        },
    ]
});

// Payload type extraction patterns for Python FastAPI handlers
static RESPONSE_MODEL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"response_model\s*=\s*([A-Za-z_][\w.\[\], |]*\w[\]]?)").unwrap()
});

static BODY_PARAM_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\w+)\s*:\s*([A-Za-z_][\w.\[\]| ]*?)\s*=\s*Body\(").unwrap()
});

static RETURN_TYPE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\)\s*->\s*([A-Za-z_][\w.\[\], |]*\w[\]]?)\s*:").unwrap()
});

// Also match `schemas.X` parameters without Body()
static SCHEMA_PARAM_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\w+)\s*:\s*(schemas\.\w[\w.\[\]| ]*)").unwrap()
});

const PRIMITIVES: &[&str] = &[
    "str", "int", "float", "dict", "list", "none", "bool", "any", "bytes", "object",
    "string", "number", "void", "undefined", "optional", "union",
];

/// Normalize a raw type string into clean type names.
/// Strips module prefixes, unwraps generics, handles unions, filters primitives.
fn normalize_types(raw: &str) -> Vec<String> {
    let mut results = Vec::new();

    // Split on | for union types
    for part in raw.split('|') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Unwrap generics: list[schemas.Message] -> schemas.Message, Page[Peer] -> Peer
        let inner = if let Some(bracket_start) = trimmed.find('[') {
            if let Some(bracket_end) = trimmed.rfind(']') {
                &trimmed[bracket_start + 1..bracket_end]
            } else {
                trimmed
            }
        } else {
            trimmed
        };

        // Split on comma for multi-arg generics
        for item in inner.split(',') {
            let item = item.trim();
            // Strip module prefix: schemas.Conclusion -> Conclusion
            let name = item.rsplit('.').next().unwrap_or(item).trim();

            if !name.is_empty() && !PRIMITIVES.contains(&name.to_lowercase().as_str()) {
                // Check it starts with uppercase (likely a type name)
                if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    results.push(name.to_string());
                }
            }
        }
    }

    results.sort();
    results.dedup();
    results
}

/// Extract consumes/produces payload types from the handler body around a transport definition.
fn extract_payload_types(content: &str, match_start: usize) -> (Option<Vec<String>>, Option<Vec<String>>) {
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = content[..match_start].lines().count();
    let start = if line_idx > 0 { line_idx - 1 } else { 0 };
    let end = (start + 30).min(lines.len());
    let snippet = lines[start..end].join("\n");

    let mut consumes = Vec::new();
    let mut produces = Vec::new();

    // Response model → produces
    if let Some(cap) = RESPONSE_MODEL_RE.captures(&snippet) {
        produces.extend(normalize_types(&cap[1]));
    }

    // Body parameter → consumes
    if let Some(cap) = BODY_PARAM_RE.captures(&snippet) {
        consumes.extend(normalize_types(&cap[2]));
    }

    // schemas.X parameter fallback → consumes
    if consumes.is_empty() {
        for cap in SCHEMA_PARAM_RE.captures_iter(&snippet) {
            consumes.extend(normalize_types(&cap[2]));
        }
    }

    // Return type annotation fallback → produces
    if produces.is_empty() {
        if let Some(cap) = RETURN_TYPE_RE.captures(&snippet) {
            produces.extend(normalize_types(&cap[1]));
        }
    }

    consumes.sort();
    consumes.dedup();
    produces.sort();
    produces.dedup();

    let consumes = if consumes.is_empty() { None } else { Some(consumes) };
    let produces = if produces.is_empty() { None } else { Some(produces) };

    (consumes, produces)
}

static GRPC_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^service\s+(\w+)\s*\{").unwrap()
});

static WEBSOCKET_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r#"(?m)(?:WebSocket|ws|io)\s*\.\s*on\s*\(\s*['"](\w+)['"]"#).unwrap(),
        Regex::new(r"(?m)@WebSocketGateway|@SubscribeMessage").unwrap(),
    ]
});

pub fn detect_transports(
    content: &str,
    language: &str,
    file: &str,
) -> Vec<DetectedComponent> {
    let mut components = Vec::new();

    // HTTP routes
    for pattern in HTTP_PATTERNS.iter() {
        for cap in pattern.regex.captures_iter(content) {
            let method = cap[pattern.method_group].to_uppercase();
            let path = cap[pattern.path_group].to_string();
            let display_name = format!("{method} {path}");
            let match_start = cap.get(0).unwrap().start();
            let line_num = content[..match_start].lines().count() as u32 + 1;

            let (consumes, produces) = extract_payload_types(content, match_start);

            components.push(DetectedComponent {
                id: make_id("transport", &display_name, file),
                name: display_name,
                kind: ComponentKind::Transport,
                language: language.to_string(),
                source: SourceLocation {
                    file: file.to_string(),
                    line_start: Some(line_num),
                    line_end: None,
                },
                metadata: HashMap::new(),
                transport_protocol: Some(pattern.protocol.clone()),
                http_method: Some(method),
                http_path: Some(path),
                model_fields: None,
                consumes,
                produces,
            });
        }
    }

    // gRPC services (protobuf)
    for cap in GRPC_PATTERN.captures_iter(content) {
        let name = cap[1].to_string();
        let match_start = cap.get(0).unwrap().start();
        let line_num = content[..match_start].lines().count() as u32 + 1;

        components.push(DetectedComponent {
            id: make_id("transport", &name, file),
            name,
            kind: ComponentKind::Transport,
            language: language.to_string(),
            source: SourceLocation {
                file: file.to_string(),
                line_start: Some(line_num),
                line_end: None,
            },
            metadata: HashMap::new(),
            transport_protocol: Some(TransportProtocol::Grpc),
            http_method: None,
            http_path: None,
            model_fields: None,
            consumes: None,
            produces: None,
        });
    }

    // WebSocket patterns
    for pattern in WEBSOCKET_PATTERNS.iter() {
        for cap in pattern.captures_iter(content) {
            let name = cap
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "WebSocket".to_string());
            let match_start = cap.get(0).unwrap().start();
            let line_num = content[..match_start].lines().count() as u32 + 1;

            components.push(DetectedComponent {
                id: make_id("transport", &name, file),
                name: format!("ws:{name}"),
                kind: ComponentKind::Transport,
                language: language.to_string(),
                source: SourceLocation {
                    file: file.to_string(),
                    line_start: Some(line_num),
                    line_end: None,
                },
                metadata: HashMap::new(),
                transport_protocol: Some(TransportProtocol::Websocket),
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
