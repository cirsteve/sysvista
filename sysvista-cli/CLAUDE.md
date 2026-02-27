# CLAUDE.md — sysvista-cli

System architecture scanner. Walks a codebase, detects components (models, services, transports, transforms), infers relationships between them, and outputs a JSON graph.

## Quick Reference

```bash
# Build and run
cd sysvista-cli
cargo build --release
./target/release/sysvista-cli scan /path/to/project -o output.json

# Test
cargo test                    # all 51 tests
cargo test relationships      # just relationship inference tests
```

## Project Structure

```
src/
├── main.rs                   # CLI entry — clap arg parsing, calls scanner::scan()
├── scanner/
│   ├── mod.rs                # Scan orchestration pipeline
│   ├── file_walker.rs        # Directory traversal (respects .gitignore via `ignore` crate)
│   ├── language.rs           # File extension → language string mapping
│   ├── models.rs             # Detect data structures (dataclass, BaseModel, TypedDict, struct, interface)
│   ├── services.rs           # Detect services (decorators → directory convention → class heuristic)
│   ├── transports.rs         # Detect HTTP/gRPC/WebSocket routes, extract payload types
│   ├── transforms.rs         # Detect to_*/from_*/convert* functions
│   ├── relationships.rs      # Infer edges: imports, references, calls, persists, handles, etc.
│   └── workflows.rs          # Synthesize request-handling workflows from transport entry points
└── output/
    ├── schema.rs             # Core types: DetectedComponent, DetectedEdge, ComponentKind, Workflow
    └── writer.rs             # JSON serialization
```

## Scan Pipeline

`scanner::scan()` in `mod.rs` orchestrates everything:

1. **Walk** — `file_walker::walk_directory()` collects files, respecting `.gitignore`
2. **Detect** — For each file, run all four detectors: `models`, `services`, `transports`, `transforms`
3. **Deduplicate** — Remove components with duplicate IDs
4. **Infer edges** — Three passes:
   - `infer_edges()` — import/reference edges from file-level analysis
   - `infer_flow_edges()` — semantic edges (handles, persists, transforms, consumes, produces)
   - `infer_call_edges()` — function call/dispatch edges from body scanning
5. **Synthesize workflows** — Follow call chains from transport entry points
6. **Output** — Serialize to JSON

## Key Types (output/schema.rs)

```
ComponentKind: Model | Service | Transport | Transform
DetectedComponent: { id, name, kind, language, source, metadata, consumes, produces, ... }
DetectedEdge: { from_id, to_id, label, payload_type }
Workflow: { id, name, entry_point_id, steps: [{ component_id, step_type, order }] }
```

Component IDs are deterministic: SHA256 of `"kind:name:file"`, first 16 hex chars.

## Edge Labels

| Label | Meaning | Produced by |
|---|---|---|
| `imports` | File-level import statement | `infer_edges` |
| `references` | Type name appears in file | `infer_edges` |
| `handles` | Service → transport in same file | `infer_flow_edges` |
| `persists` | Component body references a model type | `infer_flow_edges` |
| `transforms` | Transform body references a model type | `infer_flow_edges` |
| `consumes` | Transport declares input payload type | `infer_flow_edges` |
| `produces` | Transport declares output payload type | `infer_flow_edges` |
| `calls` | Body contains `module.func()` or `await func()` | `infer_call_edges` |
| `dispatches` | Body contains `background_tasks.add_task(func)` | `infer_call_edges` |

## Patterns and Conventions

### Regex Detection

Each scanner module defines `LazyLock<Regex>` or `LazyLock<Vec<Regex>>` statics for language-specific patterns. These compile once and are reused across all files.

```rust
static PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"...").unwrap());
```

### Body Window Scanning

Relationship inference reads a fixed line window from each component's `line_start`:
- Transports: 80 lines (handler bodies are short)
- Services: 150 lines (class bodies are larger)
- Transforms: 50 lines

### Service Detection Fallback Chain

Services use a three-tier detection strategy, tracked in `metadata["detection"]`:
1. **Decorator-based** — `@Controller`, `@Injectable`, `@Service`, etc.
2. **Directory convention** — files in `services/`, `controllers/`, `crud/`, etc.
3. **Class heuristic** — Python classes not inheriting from model bases

### Test Helpers

Tests use `make_comp()` to build components and `HashMap<String, String>` for file contents:

```rust
fn make_comp(id: &str, name: &str, kind: ComponentKind, file: &str, line: u32) -> DetectedComponent
```

Tests are co-located in `#[cfg(test)] mod tests` blocks within each module.

## Dependencies

| Crate | Purpose |
|---|---|
| `clap` (derive) | CLI argument parsing |
| `serde` + `serde_json` | JSON serialization |
| `regex` | Pattern matching for component detection |
| `ignore` | .gitignore-aware directory walking |
| `sha2` | Deterministic component ID generation |
| `chrono` | Timestamps in scan output |

## Adding a New Scanner

1. Add variant to `ComponentKind` in `output/schema.rs`
2. Create `scanner/new_kind.rs` with `pub fn detect_*(content, language, file) -> Vec<DetectedComponent>`
3. Register in `scanner/mod.rs` — add module declaration and call in the scan loop
4. Add relationship inference if needed in `relationships.rs`
5. Add tests with `make_comp()` helpers

## Style

- Iterator chains for linear transforms, imperative loops when accumulating multiple outputs or branching
- `unwrap()` only in `LazyLock` regex compilation (provably infallible) and tests
- Graceful fallbacks with `unwrap_or()` / `unwrap_or_default()` in scan logic — never panic on bad input
- Functions are pure where possible: take `&str`/`&[T]` inputs, return `Vec<T>` outputs
