# SysVista

Scan codebases and produce interactive visual maps of system architecture. Quickly understand a system's data models, services, transports, and transforms, then drill down into specifics.

**Architecture:** Rust CLI scanner → JSON → React/TypeScript web viewer

## Prerequisites

- [Rust](https://rustup.rs/) (1.85+)
- [Node.js](https://nodejs.org/) (20+)

## Install

```bash
# Clone the repo
git clone <repo-url> && cd sysvista

# Build the CLI
cd sysvista-cli
cargo build --release
# Binary is at target/release/sysvista-cli

# Install web viewer dependencies
cd ../sysvista-web
npm install
```

Or use the Makefile shortcuts:

```bash
make build-cli    # cargo build --release
make build-web    # npm run build (production bundle)
```

## Usage

### 1. Scan a project

```bash
# From sysvista-cli/
cargo run -- scan /path/to/your/project -o output.json

# Or with the release binary
./target/release/sysvista-cli scan /path/to/your/project -o output.json
```

Options:

```
Usage: sysvista-cli scan [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to the project root

Options:
  -o, --output <OUTPUT>  Output JSON file path [default: sysvista-output.json]
```

The scanner respects `.gitignore` rules and skips hidden files automatically.

### 2. View the architecture

```bash
cd sysvista-web
npm run dev
# Opens at http://localhost:5173
```

Then either:
- Click **Load JSON** in the toolbar and pick your `output.json`
- Drag and drop a JSON file onto the page
- Click **Sample** to load the bundled example

### 3. Scan directly into the viewer

```bash
make scan TARGET=/path/to/your/project
make dev-web
```

This writes the scan output to `sysvista-web/public/sample-output.json` so the viewer can load it with the Sample button.

## What it detects

| Category | Patterns |
|---|---|
| **Models** | `interface`, `type`, `enum`, `struct`, `@dataclass class`, `class X(BaseModel)`, protobuf `message` |
| **Services** | `@Controller`, `@RestController`, `@Injectable`, `@Service`, classes in `services/`/`controllers/`/`handlers/` dirs |
| **Transports** | `router.get("/path")`, `@Get("/path")`, `@app.get("/path")`, `@GetMapping`, gRPC `service` blocks, WebSocket handlers |
| **Transforms** | Functions named `to_*`, `from_*`, `convert*`, `transform*`; Rust `impl From<A> for B` |
| **Edges** | Import/require/use statements, type name references across files |
| **Flow edges** | `handles` (service → transport in same file), `persists` (transport → model referenced in handler body), `transforms` (transform → model referenced in body) |

### Supported languages

TypeScript, JavaScript, Rust, Python, Go, Java, Kotlin, C#, Ruby, Protobuf, GraphQL

## Web viewer features

- **System view** — full graph of all components and edges (pan, zoom, minimap, dagre auto-layout)
- **Flow view** — curated left-to-right data flow graph showing only components connected by flow edges (handles, persists, calls, dispatches, etc.), stripping away structural noise like imports/references
- **Smart clustering** — components grouped by semantic prefix (e.g. all "Session*" models together), with cluster headers
- **Hub highlighting** — high-degree nodes get a ring glow + degree badge; amber in the minimap
- **Workflow trace** — select a detected workflow to highlight its step components in the flow graph
- **Color-coded nodes** — blue (model), green (service), orange (transport), purple (transform)
- **Color-coded edges** — pink (payload), green (calls), amber (dispatches), cyan (other flow)
- **Click to inspect** — detail panel shows source location, language, fields, metadata, connected components
- **Fuzzy search** — find components by name, file path, or HTTP path; auto-switches to system view if component isn't in flow view
- **Filter chips** — toggle component kinds on/off
- **Drag-and-drop** — load JSON files without any server

See [`sysvista-web/USAGE.md`](sysvista-web/USAGE.md) for detailed web viewer usage.

## JSON schema

The CLI produces and the viewer consumes a `SysVistaOutput` JSON document:

```jsonc
{
  "version": "1",
  "scanned_at": "2026-02-13T12:00:00Z",
  "root_dir": "/path/to/project",
  "project_name": "my-project",
  "detected_languages": ["typescript", "rust"],
  "components": [
    {
      "id": "a1b2c3d4e5f60001",
      "name": "UserService",
      "kind": "service",            // "model" | "service" | "transport" | "transform"
      "language": "typescript",
      "source": { "file": "src/services/user.service.ts", "line_start": 8 },
      "metadata": {},
      "transport_protocol": null,    // "http" | "grpc" | "websocket" | "mq" | "graphql"
      "http_method": null,
      "http_path": null,
      "model_fields": null           // ["id", "name", "email"] for models
    }
  ],
  "edges": [
    { "from_id": "...", "to_id": "...", "label": "imports" }
  ],
  "scan_stats": {
    "files_scanned": 42,
    "files_skipped": 3,
    "scan_duration_ms": 87
  }
}
```

## Project structure

```
sysvista/
  Makefile
  sysvista-cli/                   # Rust crate
    src/
      main.rs                     # CLI entrypoint (clap)
      scanner/                    # Detection heuristics
        mod.rs                    # Orchestrator
        file_walker.rs            # .gitignore-aware directory walking
        language.rs               # Language detection by extension
        models.rs                 # Struct/interface/type detection
        services.rs               # Controller/handler detection
        transports.rs             # HTTP route/gRPC/WebSocket detection
        transforms.rs             # Conversion function detection
        relationships.rs          # Edge inference from imports + references
      output/
        schema.rs                 # Serde structs (JSON contract)
        writer.rs                 # JSON file output
  sysvista-web/                   # React/TypeScript (Vite)
    USAGE.md                      # Detailed web viewer usage guide
    src/
      types/schema.ts             # TypeScript types (JSON contract)
      lib/
        loader.ts                 # File picker + drag-and-drop
        graph-adapter.ts          # Schema → React Flow nodes/edges (buildGraph + buildFlowGraph)
        clustering.ts             # Semantic clustering + hub detection
        search.ts                 # Fuzzy search (Fuse.js)
      hooks/useGraphData.ts       # Main state management, view modes, workflow tracing
      components/
        GraphCanvas.tsx            # React Flow wrapper (shared by system + flow views)
        DetailPanel.tsx            # Slide-in component inspector
        SearchBar.tsx              # Search + filter chips
        Toolbar.tsx                # Load JSON, flow view toggle, fit view
        Legend.tsx                  # Color/shape legend (adapts to view mode)
        WorkflowPanel.tsx          # Workflow list + trace highlight panel
        nodes/                     # Custom node components per kind (LR/TB handle support)
```
