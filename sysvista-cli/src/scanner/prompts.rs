use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use crate::output::schema::{ComponentKind, DetectedComponent, SourceLocation};

use super::make_id;

// ---------------------------------------------------------------------------
// Internal types for structural tracing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResponseHandling {
    Classifier,
    Validator,
    Extractor,
    Generator,
    Unknown,
}

#[derive(Debug)]
struct TracedPromptContext {
    enclosing_function: Option<String>,
    system_prompt_source: Option<String>,
    builder_function: Option<String>,
    response_handling: ResponseHandling,
}

// ---------------------------------------------------------------------------
// Regex patterns — structural tracing
// ---------------------------------------------------------------------------

/// SDK API call: *.chat.completions.create / *.messages.create
static SDK_API_CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)(\w+)\s*=\s*(?:await\s+)?(?:\w+\.)+(?:chat\.completions|messages)\.create\(",
    )
    .unwrap()
});

/// Instructor pattern: response_model= inside a chat.completions.create call
static INSTRUCTOR_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)(\w+)\s*=\s*(?:await\s+)?(?:\w+\.)+chat\.completions\.create\([^)]*response_model\s*=",
    )
    .unwrap()
});

/// Python function definition — for finding enclosing function
static PYTHON_DEF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(?:async\s+)?def\s+(\w+)\s*\(").unwrap()
});

/// Python class definition — for __init__ fallback
static PYTHON_CLASS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^class\s+(\w+)").unwrap()
});

/// system= keyword argument
static SYSTEM_KWARG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"system\s*=\s*([^,\n]+)").unwrap()
});

/// get_prompt() call — extract argument
static GET_PROMPT_CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"get_prompt\s*\(\s*([^)]+?)\s*\)").unwrap()
});

/// Builder function call pattern
static BUILDER_CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(_?(?:build|make|create|format|render)_\w*prompt\w*)\s*\(").unwrap()
});

/// JSON parse patterns in response handling
static JSON_PARSE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:json\.loads|\.json\(\)|_parse_\w+|JSON\.parse)").unwrap()
});

/// Plain text extraction patterns
static PLAIN_TEXT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\.(?:content\[0\]\.)?text(?:\.strip\(\))?").unwrap()
});

/// Classifier field names in response handling
static CLASSIFIER_FIELDS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"["'](?:score|relevance_score|relevant|rating|confidence|label|category|classification)["']"#,
    )
    .unwrap()
});

/// Validator field names in response handling
static VALIDATOR_FIELDS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"["'](?:verdict|approved?|reject(?:ed)?|pass(?:ed)?|fail(?:ed)?|valid|feedback|quality_score)["']"#,
    )
    .unwrap()
});

/// Builder function definition — used to identify lines covered by builder defs
static BUILDER_DEF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^(?:async\s+)?def\s+(_?(?:build|make|create|format|render)_\w*prompt\w*)\s*\(",
    )
    .unwrap()
});

// ---------------------------------------------------------------------------
// Framework patterns — these already produce good names
// ---------------------------------------------------------------------------

/// A single prompt detection rule: a regex, which capture group holds the name,
/// and an optional fixed subtype. When `subtype` is None the detector falls back
/// to `infer_subtype()`.
struct PromptPattern {
    regex: Regex,
    name_group: usize,
    subtype: Option<&'static str>,
}

static FRAMEWORK_PATTERNS: LazyLock<Vec<PromptPattern>> = LazyLock::new(|| {
    vec![
        // --- LangChain (Python) ---
        PromptPattern {
            regex: Regex::new(
                r"(?m)(\w+)\s*=\s*ChatPromptTemplate\.from_(?:messages|template)\(",
            )
            .unwrap(),
            name_group: 1,
            subtype: None,
        },
        // PromptTemplate(...)
        PromptPattern {
            regex: Regex::new(r"(?m)(\w+)\s*=\s*PromptTemplate\(").unwrap(),
            name_group: 1,
            subtype: None,
        },
        // LangChain hub.pull("...")
        PromptPattern {
            regex: Regex::new(r#"(?m)(\w+)\s*=\s*hub\.pull\(\s*["']([^"']+)["']"#).unwrap(),
            name_group: 1,
            subtype: None,
        },
        // --- TypeScript / JavaScript ---
        PromptPattern {
            regex: Regex::new(
                r"(?m)(?:const|let|var)\s+(\w+)\s*=\s*new\s+(?:Chat)?PromptTemplate\(",
            )
            .unwrap(),
            name_group: 1,
            subtype: None,
        },
        // LangChain JS: ChatPromptTemplate.fromMessages / .fromTemplate
        PromptPattern {
            regex: Regex::new(
                r"(?m)(?:const|let|var)\s+(\w+)\s*=\s*ChatPromptTemplate\.from(?:Messages|Template)\(",
            )
            .unwrap(),
            name_group: 1,
            subtype: None,
        },
        // --- Semantic Kernel (.NET / Python) ---
        PromptPattern {
            regex: Regex::new(r#"(?m)(\w+)\s*=\s*kernel\.create_function_from_prompt\("#).unwrap(),
            name_group: 1,
            subtype: None,
        },
        // --- Decorator / annotation patterns ---
        PromptPattern {
            regex: Regex::new(
                r"(?m)@(?:prompt|prompt_template|llm_prompt)(?:\([^)]*\))?\s*\n\s*(?:async\s+)?def\s+(\w+)",
            )
            .unwrap(),
            name_group: 1,
            subtype: None,
        },
        // --- DSPy ---
        PromptPattern {
            regex: Regex::new(
                r"(?m)^class\s+(\w+)\(dspy\.(?:Signature|Module|ChainOfThought|Predict)\)",
            )
            .unwrap(),
            name_group: 1,
            subtype: None,
        },
        // --- Guidance / LMQL ---
        PromptPattern {
            regex: Regex::new(
                r"(?m)@guidance\s*(?:\([^)]*\))?\s*\n\s*(?:async\s+)?def\s+(\w+)",
            )
            .unwrap(),
            name_group: 1,
            subtype: None,
        },
    ]
});

// ---------------------------------------------------------------------------
// Subtype keyword table — maps name fragments to prompt subtypes.
// Checked against the name first, then the file path as fallback.
// First match wins. "generator" is the default when nothing matches.
// ---------------------------------------------------------------------------
const SUBTYPE_KEYWORDS: &[(&[&str], &str)] = &[
    (
        &["route", "router", "routing", "dispatch", "triage"],
        "router",
    ),
    (
        &[
            "classif", "categoriz", "label", "detect", "intent", "eval", "score", "relevance",
            "filter", "rank", "assess",
        ],
        "classifier",
    ),
    (
        &["extract", "parse", "structur", "entity"],
        "extractor",
    ),
    (
        &["summar", "digest", "condense", "tldr", "recap"],
        "summarizer",
    ),
    (
        &[
            "valid", "check", "verify", "guard", "assert", "comply", "critic", "critique",
            "review", "feedback", "quality",
        ],
        "validator",
    ),
    // generator is the fallback — anything that doesn't match above
];

fn match_keywords(text: &str) -> Option<&'static str> {
    let lower = text.to_lowercase();
    for &(keywords, subtype) in SUBTYPE_KEYWORDS {
        if keywords.iter().any(|kw| lower.contains(kw)) {
            return Some(subtype);
        }
    }
    None
}

/// Infer prompt subtype from the component name, falling back to the file path.
fn infer_subtype(name: &str, file: &str) -> &'static str {
    match_keywords(name)
        .or_else(|| {
            let file_stem = std::path::Path::new(file)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            match_keywords(file_stem)
        })
        .unwrap_or("generator")
}

// ---------------------------------------------------------------------------
// Core structural tracing
// ---------------------------------------------------------------------------

/// Given file content and the line number of an SDK API call, trace backward
/// to find the enclosing function and forward to classify response handling.
fn trace_sdk_call(content: &str, api_call_line: usize) -> TracedPromptContext {
    let lines: Vec<&str> = content.lines().collect();
    let call_idx = api_call_line.saturating_sub(1); // 1-based → 0-based

    // --- Lookback: find enclosing function (50 lines before) ---
    let lookback_start = call_idx.saturating_sub(50);
    let mut enclosing_function: Option<String> = None;
    let mut enclosing_class: Option<String> = None;

    // Scan backward from the API call line
    for i in (lookback_start..call_idx).rev() {
        let line = lines.get(i).unwrap_or(&"");
        if enclosing_function.is_none() {
            if let Some(cap) = PYTHON_DEF.captures(line) {
                let func_name = cap[1].to_string();
                if func_name == "__init__" {
                    // Skip __init__, keep looking for class name
                    enclosing_function = None;
                } else {
                    enclosing_function = Some(func_name);
                }
            }
        }
        if let Some(cap) = PYTHON_CLASS.captures(line) {
            enclosing_class = Some(cap[1].to_string());
            break; // class found, stop looking
        }
    }

    // If enclosing function is still None (was __init__ or not found), use class name
    if enclosing_function.is_none() {
        if let Some(ref class_name) = enclosing_class {
            // Convert class name: CamelCase → snake_case style
            enclosing_function = Some(camel_to_snake(class_name));
        }
    }

    // --- Full context window: find system kwarg, get_prompt, builder call ---
    let context_start = lookback_start;
    let context_end = (call_idx + 30).min(lines.len());
    let context_window: String = lines[context_start..context_end].join("\n");

    let system_prompt_source = SYSTEM_KWARG
        .captures(&context_window)
        .and_then(|cap| {
            let val = cap[1].trim().to_string();
            if let Some(gp) = GET_PROMPT_CALL.captures(&val) {
                Some(format!("get_prompt({})", gp[1].trim()))
            } else {
                Some(val)
            }
        });

    let builder_function = BUILDER_CALL
        .captures(&context_window)
        .map(|cap| cap[1].to_string());

    // --- Lookahead: classify response handling (30 lines after) ---
    let lookahead_start = call_idx;
    let lookahead_end = (call_idx + 30).min(lines.len());
    let lookahead_window: String = lines[lookahead_start..lookahead_end].join("\n");

    let has_json_parse = JSON_PARSE_PATTERN.is_match(&lookahead_window);
    let has_classifier_fields = CLASSIFIER_FIELDS.is_match(&lookahead_window);
    let has_validator_fields = VALIDATOR_FIELDS.is_match(&lookahead_window);
    let has_plain_text = PLAIN_TEXT_PATTERN.is_match(&lookahead_window);

    let response_handling = if has_json_parse && has_classifier_fields {
        ResponseHandling::Classifier
    } else if has_json_parse && has_validator_fields {
        ResponseHandling::Validator
    } else if has_json_parse {
        ResponseHandling::Extractor
    } else if has_plain_text && !has_json_parse {
        ResponseHandling::Generator
    } else {
        ResponseHandling::Unknown
    };

    TracedPromptContext {
        enclosing_function,
        system_prompt_source,
        builder_function,
        response_handling,
    }
}

/// Convert CamelCase to snake_case
fn camel_to_snake(name: &str) -> String {
    let mut result = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }
    result
}

/// Resolve subtype from structural analysis, with keyword/filename fallbacks.
fn resolve_subtype(
    response_handling: &ResponseHandling,
    function_name: Option<&str>,
    file: &str,
) -> &'static str {
    // 1. Structural signal (response handling) → strongest
    match response_handling {
        ResponseHandling::Classifier => return "classifier",
        ResponseHandling::Validator => return "validator",
        ResponseHandling::Extractor => return "extractor",
        ResponseHandling::Generator => return "generator",
        ResponseHandling::Unknown => {}
    }

    // 2. Keyword match on enclosing function name
    if let Some(fname) = function_name {
        if let Some(subtype) = match_keywords(fname) {
            return subtype;
        }
    }

    // 3. Keyword match on file stem
    let file_stem = std::path::Path::new(file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if let Some(subtype) = match_keywords(file_stem) {
        return subtype;
    }

    // 4. Default
    "generator"
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn detect_prompts(
    content: &str,
    language: &str,
    file: &str,
) -> Vec<DetectedComponent> {
    let mut components = Vec::new();
    let mut covered_lines: HashSet<u32> = HashSet::new();

    // --- Pass 1: SDK API calls — structural tracing ---
    // First, collect lines covered by builder function definitions so we can skip them
    let mut builder_def_lines: HashSet<u32> = HashSet::new();
    for cap in BUILDER_DEF.captures_iter(content) {
        let match_start = cap.get(0).unwrap().start();
        let line_num = content[..match_start].lines().count() as u32 + 1;
        // Mark the definition and a window around it as covered
        for l in line_num..line_num + 20 {
            builder_def_lines.insert(l);
        }
    }

    // Process instructor pattern first (more specific, has response_model=)
    for cap in INSTRUCTOR_PATTERN.captures_iter(content) {
        let match_start = cap.get(0).unwrap().start();
        let line_num = content[..match_start].lines().count() as u32 + 1;

        let ctx = trace_sdk_call(content, line_num as usize);
        let name = ctx
            .enclosing_function
            .clone()
            .unwrap_or_else(|| cap[1].to_string());

        let mut metadata = HashMap::new();
        metadata.insert("detection".to_string(), "sdk_structural".to_string());
        metadata.insert("response_handling".to_string(), "extractor".to_string());
        if let Some(ref src) = ctx.system_prompt_source {
            metadata.insert("system_prompt_source".to_string(), src.clone());
        }
        if let Some(ref builder) = ctx.builder_function {
            metadata.insert("builder_function".to_string(), builder.clone());
        }

        // Mark lines as covered
        let start = line_num.saturating_sub(5);
        let end = line_num + 15;
        for l in start..=end {
            covered_lines.insert(l);
        }

        components.push(DetectedComponent {
            id: make_id("prompt", &format!("{name}:{line_num}"), file),
            name,
            kind: ComponentKind::Prompt,
            language: language.to_string(),
            source: SourceLocation {
                file: file.to_string(),
                line_start: Some(line_num),
                line_end: None,
            },
            metadata,
            transport_protocol: None,
            http_method: None,
            http_path: None,
            model_fields: None,
            prompt_subtype: Some("extractor".to_string()),
            consumes: None,
            produces: None,
        });
    }

    // Process SDK API calls (skip lines already covered by instructor)
    for cap in SDK_API_CALL.captures_iter(content) {
        let match_start = cap.get(0).unwrap().start();
        let line_num = content[..match_start].lines().count() as u32 + 1;

        // Skip if already covered by instructor pattern
        if covered_lines.contains(&line_num) {
            continue;
        }

        let ctx = trace_sdk_call(content, line_num as usize);
        let name = ctx
            .enclosing_function
            .clone()
            .unwrap_or_else(|| cap[1].to_string());

        let subtype = resolve_subtype(
            &ctx.response_handling,
            ctx.enclosing_function.as_deref(),
            file,
        );

        let mut metadata = HashMap::new();
        metadata.insert("detection".to_string(), "sdk_structural".to_string());
        metadata.insert(
            "response_handling".to_string(),
            format!("{:?}", ctx.response_handling).to_lowercase(),
        );
        if let Some(ref src) = ctx.system_prompt_source {
            metadata.insert("system_prompt_source".to_string(), src.clone());
        }
        if let Some(ref builder) = ctx.builder_function {
            metadata.insert("builder_function".to_string(), builder.clone());
        }

        // Mark lines as covered
        let start = line_num.saturating_sub(5);
        let end = line_num + 15;
        for l in start..=end {
            covered_lines.insert(l);
        }

        components.push(DetectedComponent {
            id: make_id("prompt", &format!("{name}:{line_num}"), file),
            name,
            kind: ComponentKind::Prompt,
            language: language.to_string(),
            source: SourceLocation {
                file: file.to_string(),
                line_start: Some(line_num),
                line_end: None,
            },
            metadata,
            transport_protocol: None,
            http_method: None,
            http_path: None,
            model_fields: None,
            prompt_subtype: Some(subtype.to_string()),
            consumes: None,
            produces: None,
        });
    }

    // Also mark builder definition lines as covered so Pass 3 doesn't pick them up
    covered_lines.extend(&builder_def_lines);

    // --- Pass 2: Framework patterns — skip covered lines ---
    for pattern in FRAMEWORK_PATTERNS.iter() {
        for cap in pattern.regex.captures_iter(content) {
            let name = cap[pattern.name_group].to_string();
            let match_start = cap.get(0).unwrap().start();
            let line_num = content[..match_start].lines().count() as u32 + 1;

            // Skip if this line is already covered
            if covered_lines.contains(&line_num) {
                continue;
            }

            let subtype = pattern
                .subtype
                .unwrap_or_else(|| infer_subtype(&name, file));

            let mut metadata = HashMap::new();
            metadata.insert("detection".to_string(), "framework".to_string());

            components.push(DetectedComponent {
                id: make_id("prompt", &format!("{name}:{line_num}"), file),
                name,
                kind: ComponentKind::Prompt,
                language: language.to_string(),
                source: SourceLocation {
                    file: file.to_string(),
                    line_start: Some(line_num),
                    line_end: None,
                },
                metadata,
                transport_protocol: None,
                http_method: None,
                http_path: None,
                model_fields: None,
                prompt_subtype: Some(subtype.to_string()),
                consumes: None,
                produces: None,
            });
        }
    }

    components
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Existing framework tests — unchanged behavior
    // -----------------------------------------------------------------------

    #[test]
    fn detects_langchain_chat_prompt_template() {
        let content = r#"
from langchain_core.prompts import ChatPromptTemplate

route_prompt = ChatPromptTemplate.from_messages([
    ("system", "Route the user query."),
    ("human", "{input}"),
])
"#;
        let comps = detect_prompts(content, "python", "src/prompts/router.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "route_prompt");
        assert_eq!(comps[0].kind, ComponentKind::Prompt);
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("router"));
        assert_eq!(comps[0].metadata.get("detection").unwrap(), "framework");
    }

    #[test]
    fn detects_openai_completions_create() {
        let content = r#"
class TextGenerator:
    async def generate(self):
        response = await client.chat.completions.create(
            model="gpt-4",
            messages=[{"role": "user", "content": prompt}],
        )
        return response.content[0].text
"#;
        let comps = detect_prompts(content, "python", "src/llm/generate.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "generate");
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("generator"));
        assert_eq!(
            comps[0].metadata.get("detection").unwrap(),
            "sdk_structural"
        );
    }

    #[test]
    fn detects_anthropic_self_client_messages_create() {
        let content = r#"
class CommentGenerator:
    async def generate(self, result):
        response = await self.client.messages.create(
            model=self.model,
            max_tokens=300,
            system=get_prompt(self.respond_prompt),
            messages=[
                {"role": "user", "content": _build_comment_prompt(result)}
            ],
        )
        return response.content[0].text
"#;
        let comps = detect_prompts(content, "python", "comment_generator.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "generate");
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("generator"));
        assert_eq!(
            comps[0].metadata.get("system_prompt_source").unwrap(),
            "get_prompt(self.respond_prompt)"
        );
        assert_eq!(
            comps[0].metadata.get("builder_function").unwrap(),
            "_build_comment_prompt"
        );
    }

    #[test]
    fn api_call_subtype_from_file_name_classifier() {
        let content = r#"
class RelevanceFilter:
    async def evaluate(self, message):
        response = await self.client.messages.create(
            model=self.model,
            max_tokens=300,
            messages=[{"role": "user", "content": prompt}],
        )
        result = json.loads(response.content[0].text)
        return result["score"] > 0.5
"#;
        let comps = detect_prompts(content, "python", "relevance_filter.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "evaluate");
        // Structural: json.loads + "score" field → classifier
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("classifier"));
    }

    #[test]
    fn api_call_subtype_from_file_name_validator() {
        let content = r#"
class CommentCritic:
    async def critique(self, draft):
        response = await self.client.messages.create(
            model=self.model,
            max_tokens=300,
            messages=[{"role": "user", "content": prompt}],
        )
        result = json.loads(response.content[0].text)
        return result["verdict"]
"#;
        let comps = detect_prompts(content, "python", "comment_critic.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "critique");
        // Structural: json.loads + "verdict" field → validator
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("validator"));
    }

    #[test]
    fn detects_instructor_extraction() {
        let content = r#"
class EntityExtractor:
    def extract(self):
        result = client.chat.completions.create(
            model="gpt-4",
            response_model=ExtractedEntities,
            messages=[{"role": "user", "content": text}],
        )
"#;
        let comps = detect_prompts(content, "python", "src/extract.py");
        let extractors: Vec<_> = comps
            .iter()
            .filter(|c| c.prompt_subtype.as_deref() == Some("extractor"))
            .collect();
        assert!(!extractors.is_empty());
    }

    #[test]
    fn detects_dspy_signature() {
        let content = r#"
class SentimentClassifier(dspy.Signature):
    """Classify the sentiment of text."""
    text: str = dspy.InputField()
    sentiment: str = dspy.OutputField()
"#;
        let comps = detect_prompts(content, "python", "src/classify.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "SentimentClassifier");
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("classifier"));
    }

    #[test]
    fn detects_typescript_prompt_template() {
        let content = r#"
const summaryPrompt = ChatPromptTemplate.fromMessages([
  ["system", "Summarize the following text."],
  ["human", "{text}"],
]);
"#;
        let comps = detect_prompts(content, "typescript", "src/prompts/summary.ts");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "summaryPrompt");
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("summarizer"));
    }

    #[test]
    fn detects_prompt_decorator() {
        let content = r#"
@prompt_template(model="gpt-4")
async def validate_output(text: str) -> bool:
    """Check if the output meets quality standards."""
    ...
"#;
        let comps = detect_prompts(content, "python", "src/validators.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "validate_output");
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("validator"));
    }

    #[test]
    fn infers_generator_as_default_subtype() {
        let content = r#"
reply_prompt = ChatPromptTemplate.from_template("Reply to: {message}")
"#;
        let comps = detect_prompts(content, "python", "src/chat.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("generator"));
    }

    #[test]
    fn does_not_detect_get_prompt_loader() {
        let content = r#"
def get_prompt(name: str) -> str:
    """Load a prompt by name."""
    path = PROMPTS_DIR / f"{name}.md"
    return path.read_text().strip()
"#;
        let comps = detect_prompts(content, "python", "prompts/__init__.py");
        assert!(comps.is_empty());
    }

    #[test]
    fn no_false_positives_on_plain_code() {
        let content = r#"
def process(data):
    return data.upper()

class DataProcessor:
    pass
"#;
        let comps = detect_prompts(content, "python", "src/utils.py");
        assert!(comps.is_empty());
    }

    // -----------------------------------------------------------------------
    // New structural tracing tests
    // -----------------------------------------------------------------------

    #[test]
    fn structural_trace_classifier_from_response_fields() {
        let content = r#"
class RelevanceFilter:
    def __init__(self, client, model):
        self.client = client
        self.model = model

    async def evaluate(self, message):
        prompt = _build_eval_prompt(message)
        response = await self.client.messages.create(
            model=self.model,
            max_tokens=300,
            system=get_prompt(self.eval_prompt),
            messages=[{"role": "user", "content": prompt}],
        )
        result = json.loads(response.content[0].text)
        score = result["score"]
        relevant = result["relevant"]
        return RelevanceResult(score=score, relevant=relevant)
"#;
        let comps = detect_prompts(content, "python", "relevance_filter.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "evaluate");
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("classifier"));
        assert_eq!(
            comps[0].metadata.get("response_handling").unwrap(),
            "classifier"
        );
        assert_eq!(
            comps[0].metadata.get("system_prompt_source").unwrap(),
            "get_prompt(self.eval_prompt)"
        );
        assert_eq!(
            comps[0].metadata.get("builder_function").unwrap(),
            "_build_eval_prompt"
        );
    }

    #[test]
    fn structural_trace_generator_from_plain_text() {
        let content = r#"
class CommentGenerator:
    async def generate(self, result):
        response = await self.client.messages.create(
            model=self.model,
            max_tokens=300,
            system=get_prompt(self.respond_prompt),
            messages=[
                {"role": "user", "content": _build_comment_prompt(result)}
            ],
        )
        return response.content[0].text.strip()
"#;
        let comps = detect_prompts(content, "python", "comment_generator.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "generate");
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("generator"));
        assert_eq!(
            comps[0].metadata.get("response_handling").unwrap(),
            "generator"
        );
    }

    #[test]
    fn structural_trace_validator_from_verdict_field() {
        let content = r#"
class CommentCritic:
    async def critique(self, result, draft):
        response = await self.client.messages.create(
            model=self.model,
            max_tokens=500,
            system=get_prompt(self.critique_prompt),
            messages=[
                {"role": "user", "content": _build_critique_prompt(result, draft)}
            ],
        )
        parsed = json.loads(response.content[0].text)
        verdict = parsed["verdict"]
        feedback = parsed["feedback"]
        return CritiqueResult(verdict=verdict, feedback=feedback)
"#;
        let comps = detect_prompts(content, "python", "comment_critic.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "critique");
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("validator"));
        assert_eq!(
            comps[0].metadata.get("response_handling").unwrap(),
            "validator"
        );
    }

    #[test]
    fn structural_trace_falls_back_to_keywords() {
        let content = r#"
class TextFilter:
    async def classify_text(self, text):
        response = await self.client.messages.create(
            model=self.model,
            messages=[{"role": "user", "content": text}],
        )
        return response
"#;
        // No JSON parse, no .text → Unknown handling
        // "classify" in function name → classifier via keyword fallback
        let comps = detect_prompts(content, "python", "filter.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "classify_text");
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("classifier"));
        assert_eq!(
            comps[0].metadata.get("response_handling").unwrap(),
            "unknown"
        );
    }

    #[test]
    fn structural_trace_defaults_to_generator() {
        let content = r#"
class Agent:
    async def run(self, input):
        response = await self.client.messages.create(
            model=self.model,
            messages=[{"role": "user", "content": input}],
        )
        return response
"#;
        // No JSON, no .text → Unknown; "run" has no keyword; "agent.py" has no keyword → default generator
        let comps = detect_prompts(content, "python", "agent.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "run");
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("generator"));
    }

    #[test]
    fn builder_functions_not_emitted_as_components() {
        let content = r#"
class CommentGenerator:
    async def generate(self, result):
        response = await self.client.messages.create(
            model=self.model,
            max_tokens=300,
            messages=[
                {"role": "user", "content": _build_comment_prompt(result)}
            ],
        )
        return response.content[0].text.strip()

def _build_comment_prompt(result):
    """Build the prompt for generating an engagement comment."""
    return f"Draft a comment for: {result.message.content}"
"#;
        let comps = detect_prompts(content, "python", "comment_generator.py");
        // Only the API call component, NOT the builder function
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "generate");
        assert_eq!(
            comps[0].metadata.get("builder_function").unwrap(),
            "_build_comment_prompt"
        );
    }

    #[test]
    fn framework_patterns_still_work() {
        let content = r#"
from langchain_core.prompts import ChatPromptTemplate

summary_prompt = ChatPromptTemplate.from_messages([
    ("system", "Summarize the following."),
    ("human", "{text}"),
])
"#;
        let comps = detect_prompts(content, "python", "src/summarize.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "summary_prompt");
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("summarizer"));
        assert_eq!(comps[0].metadata.get("detection").unwrap(), "framework");
    }

    #[test]
    fn openai_pattern_traces_structurally() {
        let content = r#"
class ContentClassifier:
    def classify(self, text):
        response = client.chat.completions.create(
            model="gpt-4",
            messages=[{"role": "user", "content": text}],
        )
        data = json.loads(response.choices[0].message.content)
        label = data["label"]
        confidence = data["confidence"]
        return {"label": label, "confidence": confidence}
"#;
        let comps = detect_prompts(content, "python", "classifier.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "classify");
        // json.loads + "label"/"confidence" → classifier
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("classifier"));
        assert_eq!(
            comps[0].metadata.get("detection").unwrap(),
            "sdk_structural"
        );
    }

    #[test]
    fn two_api_calls_in_same_function_get_distinct_ids() {
        // Calls must be >20 lines apart to avoid the covered_lines overlap
        // that prevents double-detection of the same call site.
        let content = r#"
class DualGenerator:
    async def generate(self, text):
        first = await self.client.messages.create(
            model=self.model,
            max_tokens=300,
            messages=[{"role": "user", "content": text}],
        )
        summary = first.content[0].text
        step_a = do_something(summary)
        step_b = do_more(step_a)
        step_c = transform(step_b)
        step_d = validate(step_c)
        step_e = enrich(step_d)
        step_f = format_output(step_e)
        step_g = finalize(step_f)
        step_h = post_process(step_g)
        step_i = clean(step_h)
        step_j = prepare(step_i)
        step_k = augment(step_j)
        step_l = normalize(step_k)
        refined = step_l

        second = await self.client.messages.create(
            model=self.model,
            max_tokens=300,
            messages=[{"role": "user", "content": refined}],
        )
        return second.content[0].text
"#;
        let comps = detect_prompts(content, "python", "dual_generator.py");
        assert_eq!(comps.len(), 2, "both API calls should be detected");
        // Both should share the same enclosing function name
        assert_eq!(comps[0].name, "generate");
        assert_eq!(comps[1].name, "generate");
        // But their IDs must differ (line number disambiguates)
        assert_ne!(comps[0].id, comps[1].id);
    }

    #[test]
    fn init_falls_back_to_class_name() {
        let content = r#"
class RelevanceScorer:
    def __init__(self, client):
        self.client = client
        response = self.client.messages.create(
            model="claude-3",
            messages=[{"role": "user", "content": "test"}],
        )
        return response.content[0].text
"#;
        let comps = detect_prompts(content, "python", "scorer.py");
        assert_eq!(comps.len(), 1);
        // __init__ skipped, class name used: RelevanceScorer → relevance_scorer
        assert_eq!(comps[0].name, "relevance_scorer");
    }
}
