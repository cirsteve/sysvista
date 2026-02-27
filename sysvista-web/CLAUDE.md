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
    ├── GraphCanvas.tsx            # React Flow wrapper — renders nodes/edges, handles interactions
    ├── Toolbar.tsx                # File loading, view toggle, fit controls
    ├── SearchBar.tsx              # Fuzzy search + component kind filter chips
    ├── DetailPanel.tsx            # Right sidebar — selected component info, connected components
    ├── WorkflowPanel.tsx          # Left sidebar (flow mode) — workflow list, step highlighting
    ├── Legend.tsx                  # Edge/node color key
    └── nodes/
        ├── ModelNode.tsx           # Blue — data structures
        ├── ServiceNode.tsx         # Green — business logic
        ├── TransportNode.tsx       # Orange — HTTP/gRPC/WS routes
        ├── TransformNode.tsx       # Purple — data transforms
        ├── ClusterLabelNode.tsx    # Cluster group header
        └── GroupLabelNode.tsx      # Kind group header (grid layout)
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
- `selectedNode` / `selectNode()` — currently inspected component
- `viewMode` / `toggleFlowView()` — "graph" or "flow"
- `selectedWorkflow` / `selectWorkflow()` — highlighted workflow in flow mode

Graph building (`buildGraph`/`buildFlowGraph`) runs inside `useMemo` — recomputes only when schema, activeKinds, or viewMode change.

## Component Conventions

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
