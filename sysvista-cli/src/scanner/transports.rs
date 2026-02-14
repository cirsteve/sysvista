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
            });
        }
    }

    components
}
