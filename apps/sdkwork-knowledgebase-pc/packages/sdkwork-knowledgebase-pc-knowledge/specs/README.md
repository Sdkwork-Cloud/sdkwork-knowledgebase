# @sdkwork/knowledgebase-pc-knowledge

Host-managed embed surface for SDKWork Knowledgebase PC.

## Public exports

| Export | Role |
| --- | --- |
| `KnowledgeView` | Full knowledge workspace for sidebar tabs and host routes |
| `KnowledgebaseModal` | Reusable modal shell for browser and desktop hosts |
| `KnowledgebaseHostSurface` | Inline or iframe presentation body |
| `configureKnowledgebasePcRuntime` | Host `sdkPorts` wiring entrypoint |
| `openKnowledgebaseDesktopWindow` | Opens a host-provided detached desktop window through `sdkPorts.openHostKnowledgeWindow` |

## Host integration

1. Bootstrap host session/SDK ports through `configureKnowledgebasePcRuntime({ sdkPorts })`.
2. Provide optional `openHostKnowledgeWindow` when the host supports native detached windows (IM desktop Tauri).
3. Expose `/host-embed/knowledge` in the host router for same-origin iframe and desktop window loads.
4. Render `KnowledgebaseModal` from chat or agent surfaces with optional `context.groupId` / `context.groupName`.

## Presentation modes

Controlled by `VITE_SDKWORK_KNOWLEDGEBASE_HOST_PRESENTATION_MODE`:

| Mode | Browser | Desktop |
| --- | --- | --- |
| `inline` | Modal embeds `KnowledgeView` directly | Modal embeds `KnowledgeView` directly |
| `detached-iframe` (desktop default) | Modal embeds `KnowledgeView` directly | Modal iframe loads `/host-embed/knowledge` |
| `detached-window` | N/A unless host provides window bridge | Host opens native window immediately |

Machine contract: `specs/component.spec.json`.
