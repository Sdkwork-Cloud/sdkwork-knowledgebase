# Component Specs

This directory is the local SDKWork component contract for `sdkwork-knowledgebase-standalone-gateway`.

- Component root: `sdkwork-knowledgebase/crates/sdkwork-knowledgebase-standalone-gateway`
- Canonical standards: `../../../sdkwork-specs/README.md`
- Machine-readable contract: `specs/component.spec.json`

Read `specs/component.spec.json` before changing this component's public exports, runtime entrypoints, SDK clients, generated artifacts, config keys, or verification commands.

Do not copy root standards into this directory. Link to files under `../../../sdkwork-specs/` instead.

## Runtime Limits And Shutdown

The standalone gateway owns every accepted Hyper HTTP/1.1 connection in a bounded task set. It
stops accepting at the configured connection limit, reaps completed connection tasks during
normal operation, and on shutdown stops admission before draining active requests. When the drain
deadline expires, remaining connection tasks are aborted and reaped before agent runtime cleanup
starts. TLS and HTTP/2 terminate at the approved platform ingress; this component does not declare
or silently enable a WebSocket/HTTP upgrade capability.

| Variable | Default | Validation |
| --- | --- | --- |
| `SDKWORK_KNOWLEDGEBASE_ENVIRONMENT` | required | Must be `development`, `test`, `staging`, or `production`; missing and non-canonical values fail startup. |
| `SDKWORK_KNOWLEDGEBASE_GATEWAY_DRAIN_TIMEOUT_SECS` | `30` | Positive integer; production requires `5` through `300`. |
| `SDKWORK_KNOWLEDGEBASE_GATEWAY_HEADER_READ_TIMEOUT_SECS` | `10` | Integer from `1` through `30`; closes slow or idle partial-header connections. |
| `SDKWORK_KNOWLEDGEBASE_GATEWAY_MAX_CONNECTIONS` | `4096` | Integer from `1` through `16384`; bounds the number of owned connection tasks. |

An invalid value fails before listener bind. A process supervisor termination grace period must
exceed the HTTP drain deadline plus runtime cleanup time; production rollout evidence must verify
that relationship before publication is enabled. Handler futures must remain cancellation-safe and
must move blocking work through the bounded blocking adapters. Tokio cannot preempt a task that
executes a non-yielding synchronous loop on an async worker; the process supervisor's termination
deadline is the final fail-stop boundary for that programming error. Signal-source setup failures
are typed: one failed source falls back to the other, while failure of every supported source
terminates serving and still runs runtime cleanup.
