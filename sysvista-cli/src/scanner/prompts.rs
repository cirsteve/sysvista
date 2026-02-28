use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::output::schema::{ComponentKind, DetectedComponent, SourceLocation};

use super::make_id;

/// A single prompt detection rule: a regex, which capture group holds the name,
/// and an optional fixed subtype. When `subtype` is None the detector falls back
/// to `infer_subtype()`.
struct PromptPattern {
    regex: Regex,
    name_group: usize,
    subtype: Option<&'static str>,
}

// ---------------------------------------------------------------------------
// All prompt detection patterns live here. Add / remove / tweak entries in
// this single vec to change what the scanner recognises as a prompt component.
// ---------------------------------------------------------------------------
static PROMPT_PATTERNS: LazyLock<Vec<PromptPattern>> = LazyLock::new(|| {
    vec![
        // --- LangChain (Python) ---
        // ChatPromptTemplate.from_messages / .from_template
        PromptPattern {
            regex: Regex::new(r"(?m)(\w+)\s*=\s*ChatPromptTemplate\.from_(?:messages|template)\(")
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

        // --- OpenAI / Anthropic / generic SDK calls ---
        // Variable assigned from *.chat.completions.create / *.messages.create
        // Uses (?:\w+\.)+ to handle chains like self.client.messages.create
        // subtype is None → inferred from variable name, then file name as fallback
        PromptPattern {
            regex: Regex::new(
                r"(?m)(\w+)\s*=\s*(?:await\s+)?(?:\w+\.)+(?:chat\.completions|messages)\.create\(",
            )
            .unwrap(),
            name_group: 1,
            subtype: None,
        },

        // --- TypeScript / JavaScript ---
        // new ChatPromptTemplate / new PromptTemplate
        PromptPattern {
            regex: Regex::new(r"(?m)(?:const|let|var)\s+(\w+)\s*=\s*new\s+(?:Chat)?PromptTemplate\(")
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

        // --- Instructor / structured output ---
        // *.chat.completions.create(..., response_model=...)
        PromptPattern {
            regex: Regex::new(
                r"(?m)(\w+)\s*=\s*(?:await\s+)?(?:\w+\.)+chat\.completions\.create\([^)]*response_model\s*=",
            )
            .unwrap(),
            name_group: 1,
            subtype: Some("extractor"),
        },

        // --- Decorator / annotation patterns ---
        // @prompt / @prompt_template / @llm_prompt
        PromptPattern {
            regex: Regex::new(
                r"(?m)@(?:prompt|prompt_template|llm_prompt)(?:\([^)]*\))?\s*\n\s*(?:async\s+)?def\s+(\w+)",
            )
            .unwrap(),
            name_group: 1,
            subtype: None,
        },

        // --- DSPy ---
        // class Foo(dspy.Signature) / class Foo(dspy.Module)
        PromptPattern {
            regex: Regex::new(r"(?m)^class\s+(\w+)\(dspy\.(?:Signature|Module|ChainOfThought|Predict)\)")
                .unwrap(),
            name_group: 1,
            subtype: None,
        },

        // --- Guidance / LMQL ---
        // @guidance decorator
        PromptPattern {
            regex: Regex::new(r"(?m)@guidance\s*(?:\([^)]*\))?\s*\n\s*(?:async\s+)?def\s+(\w+)")
                .unwrap(),
            name_group: 1,
            subtype: None,
        },

        // --- Prompt-builder functions ---
        // Python: def/async def with "prompt" in the function name
        PromptPattern {
            regex: Regex::new(r"(?m)^(?:async\s+)?def\s+((?:build|make|create|format|render)_\w*prompt\w*)\s*\(")
                .unwrap(),
            name_group: 1,
            subtype: None,
        },
        // Also match _build_*_prompt style (private helpers)
        PromptPattern {
            regex: Regex::new(r"(?m)^(?:async\s+)?def\s+(_(?:build|make|create|format|render)_\w*prompt\w*)\s*\(")
                .unwrap(),
            name_group: 1,
            subtype: None,
        },
        // TypeScript/JavaScript: export function *prompt*
        PromptPattern {
            regex: Regex::new(r"(?m)^(?:export\s+)?(?:async\s+)?function\s+((?:build|make|create|format|render)\w*[Pp]rompt\w*)\s*\(")
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
    (&["route", "router", "routing", "dispatch", "triage"], "router"),
    (&["classif", "categoriz", "label", "detect", "intent", "eval",
       "score", "relevance", "filter", "rank", "assess"], "classifier"),
    (&["extract", "parse", "structur", "entity"], "extractor"),
    (&["summar", "digest", "condense", "tldr", "recap"], "summarizer"),
    (&["valid", "check", "verify", "guard", "assert", "comply",
       "critic", "critique", "review", "feedback", "quality"], "validator"),
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
/// Generic variable names like "response" or "result" carry no semantic signal,
/// so the file name (e.g. "relevance_filter.py") often provides the real context.
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

pub fn detect_prompts(
    content: &str,
    language: &str,
    file: &str,
) -> Vec<DetectedComponent> {
    let mut components = Vec::new();

    for pattern in PROMPT_PATTERNS.iter() {
        for cap in pattern.regex.captures_iter(content) {
            let name = cap[pattern.name_group].to_string();
            let match_start = cap.get(0).unwrap().start();
            let line_num = content[..match_start].lines().count() as u32 + 1;

            let subtype = pattern.subtype.unwrap_or_else(|| infer_subtype(&name, file));

            components.push(DetectedComponent {
                id: make_id("prompt", &name, file),
                name,
                kind: ComponentKind::Prompt,
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
    }

    #[test]
    fn detects_openai_completions_create() {
        let content = r#"
response = await client.chat.completions.create(
    model="gpt-4",
    messages=[{"role": "user", "content": prompt}],
)
"#;
        // File name "generate.py" has no keyword match → falls to "generator" default
        let comps = detect_prompts(content, "python", "src/llm/generate.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "response");
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("generator"));
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
        // "response" has no keyword → file "comment_generator" has no keyword → default "generator"
        let comps = detect_prompts(content, "python", "comment_generator.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "response");
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("generator"));
    }

    #[test]
    fn api_call_subtype_from_file_name_classifier() {
        let content = r#"
        response = await self.client.messages.create(
            model=self.model,
            max_tokens=300,
            messages=[{"role": "user", "content": prompt}],
        )
"#;
        // "response" has no keyword → file "relevance_filter" contains "filter" → classifier
        let comps = detect_prompts(content, "python", "relevance_filter.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("classifier"));
    }

    #[test]
    fn api_call_subtype_from_file_name_validator() {
        let content = r#"
        response = await self.client.messages.create(
            model=self.model,
            max_tokens=300,
            messages=[{"role": "user", "content": prompt}],
        )
"#;
        // "response" has no keyword → file "comment_critic" contains "critic" → validator
        let comps = detect_prompts(content, "python", "comment_critic.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("validator"));
    }

    #[test]
    fn detects_instructor_extraction() {
        let content = r#"
result = client.chat.completions.create(
    model="gpt-4",
    response_model=ExtractedEntities,
    messages=[{"role": "user", "content": text}],
)
"#;
        let comps = detect_prompts(content, "python", "src/extract.py");
        // Should match the instructor pattern (extractor subtype)
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
    fn detects_build_prompt_function() {
        let content = r#"
def _build_comment_prompt(
    result: RelevanceResult,
    lessons: Sequence[CritiqueLesson] | None = None,
) -> str:
    """Build the prompt for generating an engagement comment."""
    prompt = f"""Draft a brief engagement comment for this Discord post.

## Original Post
- **Author:** {result.message.author_name}
- **Message:** {result.message.content}
"""
    return prompt
"#;
        // "comment" in name has no keyword → file "comment_generator" has no keyword → generator
        let comps = detect_prompts(content, "python", "comment_generator.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "_build_comment_prompt");
        assert_eq!(comps[0].kind, ComponentKind::Prompt);
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("generator"));
    }

    #[test]
    fn detects_build_eval_prompt_as_classifier() {
        let content = r#"
def _build_eval_prompt(message: Message) -> str:
    """Build evaluation prompt."""
    return f"Evaluate: {message.content}"
"#;
        // "eval" in name → classifier
        let comps = detect_prompts(content, "python", "relevance_filter.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "_build_eval_prompt");
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("classifier"));
    }

    #[test]
    fn detects_build_critique_prompt_as_validator() {
        let content = r#"
def _build_critique_prompt(result, draft: str) -> str:
    """Build the critique prompt."""
    return f"Critique this draft: {draft}"
"#;
        // "critique" in name → validator
        let comps = detect_prompts(content, "python", "comment_critic.py");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "_build_critique_prompt");
        assert_eq!(comps[0].prompt_subtype.as_deref(), Some("validator"));
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
}
