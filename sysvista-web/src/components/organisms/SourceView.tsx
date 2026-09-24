import { useEffect, useMemo, useRef, useState } from "react";
import type { SourceResult } from "../../hooks/useSource";
import type { SourceSpan } from "../../types/v2";
import { LineGutter } from "../molecules/LineGutter";

interface SourceViewProps {
  source: SourceResult;
  language?: string | null;
  span?: SourceSpan;
}

const LANGUAGES = new Set(["typescript", "javascript", "tsx", "jsx", "json", "rust", "python", "go", "java", "css", "html", "bash"]);
const aliases: Record<string, string> = { ts: "typescript", js: "javascript", rs: "rust", py: "python", sh: "bash" };
const languageLoader = (language: string) => {
  switch (language) {
    case "typescript": return import("shiki/dist/langs/typescript.mjs");
    case "javascript": return import("shiki/dist/langs/javascript.mjs");
    case "tsx": return import("shiki/dist/langs/tsx.mjs");
    case "jsx": return import("shiki/dist/langs/jsx.mjs");
    case "json": return import("shiki/dist/langs/json.mjs");
    case "rust": return import("shiki/dist/langs/rust.mjs");
    case "python": return import("shiki/dist/langs/python.mjs");
    case "go": return import("shiki/dist/langs/go.mjs");
    case "java": return import("shiki/dist/langs/java.mjs");
    case "css": return import("shiki/dist/langs/css.mjs");
    case "html": return import("shiki/dist/langs/html.mjs");
    default: return import("shiki/dist/langs/bash.mjs");
  }
};

async function highlightedHtml(text: string, language: string, span?: SourceSpan) {
  const [{ createHighlighterCore }, { createJavaScriptRegexEngine }, grammar, light, dark] = await Promise.all([
    import("shiki/core"),
    import("shiki/engine/javascript"),
    languageLoader(language),
    import("shiki/dist/themes/github-light.mjs"),
    import("shiki/dist/themes/github-dark.mjs"),
  ]);
  const highlighter = await createHighlighterCore({
    engine: createJavaScriptRegexEngine(),
    langs: grammar.default,
    themes: [light.default, dark.default],
  });
  return highlighter.codeToHtml(text, {
    lang: language,
    themes: { light: "github-light", dark: "github-dark" },
    defaultColor: false,
    transformers: [{
      line(node, line) {
        node.properties["data-line"] = line;
        if (span && line >= span.start_line && line <= span.end_line) node.properties.class = "line source-line-highlight";
      },
    }],
  });
}

export function SourceView({ source, language, span }: SourceViewProps) {
  const target = useRef<HTMLDivElement>(null);
  const [highlighted, setHighlighted] = useState<{ key: string; html: string }>();
  const selectedLanguage = aliases[language ?? ""] ?? language ?? "text";
  const text = source.kind === "text" ? source.text : "";
  const lines = useMemo(() => text.split("\n"), [text]);
  const highlightKey = `${selectedLanguage}:${span?.start_line ?? 0}:${span?.end_line ?? 0}:${text}`;
  const html = highlighted?.key === highlightKey ? highlighted.html : undefined;

  useEffect(() => {
    let active = true;
    if (!text || !LANGUAGES.has(selectedLanguage)) return;
    void highlightedHtml(text, selectedLanguage, span).then((value) => active && setHighlighted({ key: highlightKey, html: value }));
    return () => { active = false; };
  }, [highlightKey, selectedLanguage, span, text]);

  useEffect(() => {
    if (!span) return;
    requestAnimationFrame(() => target.current?.querySelector(`[data-line="${span.start_line}"]`)?.scrollIntoView({ block: "center" }));
  }, [html, span]);

  if (source.kind === "unavailable") return <p className="p-4 text-sm text-[var(--muted)]">source unavailable</p>;
  if (source.kind === "binary") return <p className="p-4 text-sm text-[var(--muted)]">binary, not rendered</p>;
  if (source.kind === "loading") return <p className="p-4 text-sm text-[var(--muted)]">Loading source…</p>;
  if (source.kind !== "text") return null;

  return (
    <div ref={target} className="grid max-h-80 grid-cols-[auto_1fr] overflow-auto border-t border-[var(--border)] text-xs">
      <LineGutter count={lines.length} highlighted={span ? { start: span.start_line, end: span.end_line } : undefined} />
      {html
        ? <div className="source-code min-w-max" dangerouslySetInnerHTML={{ __html: html }} />
        : <pre className="m-0 min-w-max bg-[var(--canvas)] px-3 py-3 font-mono leading-5"><code>{text}</code></pre>}
    </div>
  );
}
