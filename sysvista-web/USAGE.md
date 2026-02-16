# Web Viewer Usage

## Loading data

Three ways to load a SysVista scan:

1. **Toolbar** — click **Load JSON** and pick your `output.json`
2. **Drag-and-drop** — drop a JSON file anywhere on the page
3. **Public directory** — place a file at `public/sample-output.json` and load it via the Sample button (if configured)

## System view

The default view. Shows every detected component and edge in a top-to-bottom dagre layout.

### Node types

| Kind | Color | Shape |
|------|-------|-------|
| Model | Blue | Rounded rectangle |
| Service | Green | Rectangle |
| Transport | Orange | Hexagon |
| Transform | Purple | Diamond |

### Edge types

| Label | Color | Animated |
|-------|-------|----------|
| `consumes`, `produces` (payload) | Pink | Yes |
| `calls` | Green | Yes |
| `dispatches` | Amber | Yes |
| `handles`, `persists`, `transforms` (flow) | Cyan | Yes |
| `imports`, `references` (structural) | Gray | No |

### Interactions

- **Click a node** to open the detail panel (source location, language, fields, metadata, connected components)
- **Click a transport node** to auto-highlight its flow chain (the full path of connected components via flow edges)
- **Pan and zoom** with mouse/trackpad; use the minimap in the bottom-right corner
- **Fit** button resets the viewport to show all nodes

### Smart clustering

When the graph is large (>2000 edges), dagre is skipped in favor of a cluster grid layout. Components are grouped by semantic prefix (e.g. all `User*` components together), with cluster headers. Hub nodes (high-degree) appear first within each cluster.

### Hub highlighting

Nodes with many connections get visual emphasis:
- **High hubs** — amber ring glow + degree badge
- **Medium hubs** — subtle ring glow

Hubs appear amber in the minimap for quick identification.

## Flow view

Click the **Flow View** button in the toolbar to switch. This view strips away structural edges (imports, references) and shows only data-flow edges in a left-to-right layout.

### What's included

Only components that have at least one flow edge are shown. Flow edge types: `handles`, `persists`, `transforms`, `consumes`, `produces`, `calls`, `dispatches`.

### Layout

Dagre left-to-right (LR) layout. Node handles are positioned on left (input) and right (output) sides instead of top/bottom.

### Workflow highlighting

In flow view, the **Workflows** button appears in the toolbar (when workflows are detected). Opening the workflow panel and selecting a workflow highlights its step components in the flow graph. Click a step to navigate to that component.

### Search behavior

The search bar is available in both views. If you search for a component that exists in the system view but not in the flow view, the viewer automatically switches to the system view to show it.

## Search and filtering

### Fuzzy search

The search bar supports fuzzy matching against component names, file paths, and HTTP paths. Results appear as you type.

### Kind filter chips

Below the search input, toggle chips let you show/hide component kinds (model, service, transport, transform). This affects both system and flow views.

## Keyboard and mouse

| Action | Input |
|--------|-------|
| Pan | Click + drag on canvas |
| Zoom | Scroll wheel / pinch |
| Select node | Click on node |
| Close detail panel | Click X or click canvas |
| Fit view | Click **Fit** button in toolbar |
