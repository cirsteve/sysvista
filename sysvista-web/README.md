# SysVista Web Viewer

Interactive React/TypeScript viewer for SysVista scan output. Renders system architecture as explorable graphs with two view modes.

## Quick start

```bash
npm install
npm run dev        # http://localhost:5173
```

Load a scan JSON file via the toolbar button or drag-and-drop.

## Views

- **System view** (default) — full graph of all components and edges, top-to-bottom dagre layout
- **Flow view** — data-flow-only graph (left-to-right), showing only components connected by flow edges like `handles`, `persists`, `calls`, `dispatches`, `consumes`, `produces`, `transforms`

Toggle between views with the **Flow View** button in the toolbar.

## Scripts

| Command | Description |
|---------|-------------|
| `npm run dev` | Start Vite dev server |
| `npm run build` | TypeScript check + production build |
| `npm run preview` | Preview production build |
| `npm run lint` | ESLint |
| `npm test` | Vitest unit tests |

## Stack

- React 19 + TypeScript 5.9
- [React Flow](https://reactflow.dev/) (@xyflow/react) for graph rendering
- [Dagre](https://github.com/dagrejs/dagre) for auto-layout (TB for system, LR for flow)
- [Tailwind CSS](https://tailwindcss.com/) for styling
- [Fuse.js](https://www.fusejs.io/) for fuzzy search
- [Lucide](https://lucide.dev/) for icons

See [USAGE.md](USAGE.md) for detailed feature documentation.
