# @sdkwork/knowledgebase-pc-knowledge

Generic host-managed embed surface and standalone managed-group launch route for SDKWork
Knowledgebase PC.

## Public exports

| Export | Role |
| --- | --- |
| `KnowledgeView` | Full knowledge workspace for sidebar tabs and host routes |
| `KnowledgebaseModal` | Reusable modal shell for browser and desktop hosts |
| `KnowledgebaseHostSurface` | Generic inline or iframe presentation body, never a managed group workspace |
| `configureKnowledgebasePcRuntime` | Host `sdkPorts` wiring entrypoint |
| `GroupKnowledgebaseLaunchPage` | Fixed, ticket-authorized standalone group workspace route |

## Host integration

1. Bootstrap generic host session/SDK ports through `configureKnowledgebasePcRuntime({ sdkPorts })`.
2. Use generic embedding only when the host owns its own non-group context and access policy.
3. Do not render `KnowledgebaseModal`, `KnowledgebaseHostSurface`, or a host-provided detached
   window for a managed IM Conversation group.
4. Managed group launch is `/group-launch` under the Knowledgebase public web base path. It accepts
   an opaque fragment/deep-link ticket, removes it from visible browser state, consumes it after
   normal authentication, and opens only the server-authorized fixed space.

## Managed Group Boundary

The managed group route is always a full Knowledgebase application surface. Its browser flow opens
a standalone tab and its desktop flow targets the independent Knowledgebase Tauri application via
the strict `sdkwork-knowledgebase://group-launch/<opaque-ticket>` protocol. It has no inline,
iframe, or IM-owned detached-window presentation mode.

Machine contract: `specs/component.spec.json`.
