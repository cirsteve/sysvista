# CLAUDE.md — sysvista-web

Interactive architecture visualization. Loads JSON output from `sysvista-cli` and renders it as a node graph with two views: full architecture (graph mode) and data flow (flow mode).

## Quick Reference

```bash
cd sysvista-web
npm run dev       # Vite dev server
npm run build     # TypeScript check + Vite build
npm test          # Vitest (29 tests across 3 suites)
npm run lint      # ESLint
```

## Tech Stack

- **React 19** + TypeScript 5.9, Vite 7.3
- **React Flow** (`@xyflow/react`) — graph canvas, nodes, edges, layout
- **Dagre** (`@dagrejs/dagre`) — automatic graph layout (TB for graph mode, LR for flow mode)
- **Tailwind CSS 4** — all styling via utility classes, dark theme
- **Fuse.js** — fuzzy search over component names
- **Vitest** — unit tests for lib modules
- **lucide-react** — icons

## Project Structure

```
src/
├── App.tsx                       # Root SPA — ReactFlowProvider, view mode toggle, panel layout
├── main.tsx                      # Entry point — createRoot + StrictMode
├── index.css                     # Tailwind import + React Flow dark theme overrides
├── types/
│   └── schema.ts                 # TypeScript mirrors of CLI output: SysVistaOutput, DetectedComponent, DetectedEdge, Workflow
├── lib/
│   ├── design-tokens.ts          # Centralized color palette: KIND_COLORS, STEP_TYPE_COLORS, KIND_NODE_SIZE
│   ├── graph-adapter.ts          # buildGraph() + buildFlowGraph() — JSON → React Flow nodes/edges
│   ├── graph-adapter.test.ts     # Edge styling, filtering, dedup tests
│   ├── clustering.ts             # classifyComponents() + detectHubs() — grouping and hub tiers
│   ├── clustering.test.ts        # Cluster naming, hub tier thresholds
│   ├── loader.ts                 # loadFromFile(), validate(), setupDropZone()
│   ├── loader.test.ts            # Schema validation, backward compat
│   └── search.ts                 # Fuse.js index builder
├── hooks/
│   └── useGraphData.ts           # Central state hook — schema, search, selection, view mode
└── components/
    ├── atoms/                     # Smallest reusable UI primitives
    │   ├── Badge.tsx              # Labeled pill — variants: kind, count, type, step, field
    │   ├── KindDot.tsx            # Colored dot/square indicator for component kinds
    │   ├── IconButton.tsx         # Button with lucide icon + optional text + optional count badge
    │   └── SectionHeader.tsx      # Icon + label divider used in detail sections
    ├── molecules/                 # Composed atom groups
    │   ├── FilterChipGroup.tsx    # Row of kind toggle chips
    │   ├── ListItem.tsx           # Clickable row: KindDot + name + subtext + chevron
    │   ├── PanelShell.tsx         # Sidebar container — position, scroll, bg, close button
    │   └── FieldGroup.tsx         # Label + icon + children (detail sections)
    ├── organisms/                 # Full feature sections composed from atoms + molecules
    │   ├── SearchBar.tsx          # Input + FilterChipGroup + dropdown of ListItems
    │   ├── DetailPanel.tsx        # PanelShell + component info sections
    │   ├── WorkflowPanel.tsx      # PanelShell + workflow list/detail views
    │   ├── Toolbar.tsx            # Top bar — file load, view toggle, fit controls
    │   ├── Legend.tsx             # Edge/node color key
    │   └── GraphCanvas.tsx        # React Flow wrapper — nodes, edges, highlight logic
    └── nodes/                     # React Flow custom node types (outside atomic hierarchy)
        ├── ModelNode.tsx          # Blue — data structures
        ├── ServiceNode.tsx        # Green — business logic
        ├── TransportNode.tsx      # Orange — HTTP/gRPC/WS routes
        ├── TransformNode.tsx      # Purple — data transforms
        ├── ClusterLabelNode.tsx   # Cluster group header
        └── GroupLabelNode.tsx     # Kind group header (grid layout)
```

## Data Flow

1. User drops/picks a JSON file → `loader.ts` validates against `SysVistaOutput` schema
2. `useGraphData` hook stores the schema, builds a Fuse.js search index
3. `buildGraph()` or `buildFlowGraph()` transforms components + edges into React Flow format
4. `GraphCanvas` renders the result with custom node components

## Key Concepts

### Two View Modes

- **Graph mode** (`buildGraph`) — full architecture, TB (top-bottom) Dagre layout, all edge types shown. For dense graphs (>2000 edges), falls back to `clusterGridLayout()`.
- **Flow mode** (`buildFlowGraph`) — only flow-related edges, LR (left-right) layout, all edges animated. Only includes components with at least one flow edge.

### FLOW_LABELS (`graph-adapter.ts`)

```ts
const FLOW_LABELS = new Set(["handles", "persists", "transforms", "consumes", "produces", "calls", "dispatches"])
```

These labels distinguish data/control flow edges from static import/reference edges. Used to filter for flow mode and style edges differently.

### Edge Color Scheme

| Edge Type | Color | Label(s) |
|---|---|---|
| Payload | Pink `#f472b6` | consumes, produces |
| Call | Green `#22c55e` | calls |
| Dispatch | Amber `#f59e0b` | dispatches |
| Flow | Cyan `#06b6d4` | handles, persists, transforms |
| Default | Gray `#6b7280` | imports, references |

### Node Color Scheme

| Kind | Color |
|---|---|
| Model | Blue `#3b82f6` |
| Service | Green `#22c55e` |
| Transport | Orange `#f97316` |
| Transform | Purple `#a855f7` |

### Clustering (`clustering.ts`)

Groups components for grid layout on dense graphs:
- **Models/Transforms/Services** — CamelCase first word (e.g. `SessionPeerConfig` → "Session")
- **Transports** — first HTTP path segment (e.g. `/sessions/{id}` → "Sessions")
- **Fallback** — parent directory name
- Clusters with <3 members fold into "Other"

### Hub Detection (`clustering.ts`)

`detectHubs()` calculates edge degree per node and assigns tiers:
- **High** — degree > mean + 2 stddev (gets ring + degree badge)
- **Medium** — degree > mean + 1 stddev (gets subtle ring)
- **Normal** — everything else

## State Management

No Redux/Zustand. All state lives in the `useGraphData` custom hook, which returns:
- `schema` / `loadSchema()` — the loaded scan data
- `activeKinds` / `toggleKind()` — which component kinds are visible
- `searchResults` / `doSearch()` — fuzzy search matches
- `selectedNode` / `setSelectedNode()` — currently inspected component
- `viewMode` / `toggleFlowView()` — "graph" or "flow"
- `selectedWorkflow` / `selectWorkflow()` — highlighted workflow in flow mode

Graph building (`buildGraph`/`buildFlowGraph`) runs inside `useMemo` — recomputes only when schema, activeKinds, or viewMode change.

## Component Architecture (Atomic Design)

Components follow atomic design principles with three tiers:

- **Atoms** (`components/atoms/`) — Smallest reusable UI primitives. No business logic, no state. Accept simple props and render a single visual element. Examples: `Badge`, `KindDot`, `IconButton`, `SectionHeader`.
- **Molecules** (`components/molecules/`) — Compositions of 2+ atoms into a reusable UI group. May have minimal local state (e.g. toggle). Examples: `PanelShell`, `ListItem`, `FilterChipGroup`, `FieldGroup`.
- **Organisms** (`components/organisms/`) — Full feature sections that compose atoms + molecules with business logic and props from the app. These are what `App.tsx` renders. Examples: `SearchBar`, `DetailPanel`, `WorkflowPanel`, `Toolbar`, `Legend`, `GraphCanvas`.
- **Nodes** (`components/nodes/`) — React Flow custom node types. These sit outside the atomic hierarchy because they're tightly coupled to React Flow's `NodeProps`/`Handle` system and only exist in the graph context.

### Design Tokens (`lib/design-tokens.ts`)

Centralized color palette consumed across all layers:
- `KIND_COLORS` — bg, text, border, dot (Tailwind classes) + hex (for React Flow/MiniMap) per component kind
- `STEP_TYPE_COLORS` — text, bg, label per workflow step type
- `KIND_NODE_SIZE` — width/height per kind for Dagre layout

When adding a new component, classify it into the appropriate tier. If you're unsure, ask: "Does it render a single element?" (atom), "Does it compose atoms?" (molecule), or "Does it implement a feature?" (organism).

## Code Style

Bias toward `map`, `filter`, `reduce` pipelines over imperative `for` loops. Data transformations should be built from composable operations, not inline mutation.

- **Default to pipelines.** Accumulate into Maps, Sets, and grouped structures via `reduce`. Store multiple accumulators as named fields in the reducer state — this is readable and makes each accumulation independently updatable.
- **Extract named helpers** when a transform step is reusable or the pipeline gets long. Each helper should be independently testable.
- **Loops are acceptable** for graph traversal (BFS/DFS with a queue) and for calling external imperative APIs (e.g., `dagre.setNode()`). These are genuinely stateful operations, not data transforms.



- All components are functional `.tsx` with typed props interfaces
- Styling is 100% Tailwind utility classes — no CSS modules, no styled-components
- Node components receive `NodeProps` from React Flow and extract `GraphNode` from `data`
- Node handle positions adapt to layout direction: Top/Bottom for TB, Left/Right for LR
- Props use `on*` prefix for callbacks (`onClose`, `onNavigate`, `onLoad`)

## Testing

Tests live in `src/lib/*.test.ts` — pure logic tests, no component rendering tests.

- `graph-adapter.test.ts` — edge styling colors, flow filtering, deduplication, layout selection
- `clustering.test.ts` — cluster classification, hub tier assignment, fold-to-other threshold
- `loader.test.ts` — schema validation, missing fields, backward compatibility (auto-creates empty workflows)

All tests use Vitest `describe`/`it`/`expect` patterns.
