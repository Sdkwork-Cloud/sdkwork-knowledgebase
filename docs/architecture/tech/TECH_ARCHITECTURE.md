# SDKWork Knowledgebase Technical Architecture

Status: active  
Owner: SDKWork maintainers  
Updated: 2026-07-21<br>
Specs: ARCHITECTURE_DECISION_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map


- [TECH-alignment-baseline-2026-06-29.md](TECH-alignment-baseline-2026-06-29.md)
- [TECH-2026-06-11-sdkwork-structure-standardization-design.md](TECH-2026-06-11-sdkwork-structure-standardization-design.md)
- [TECH-2026-06-11-sdkwork-structure-standardization-implementation.md](TECH-2026-06-11-sdkwork-structure-standardization-implementation.md)
- [TECH-2026-06-19-okf-knowledge-bundle-design.md](TECH-2026-06-19-okf-knowledge-bundle-design.md)
- [TECH-okf-knowledge-bundle.md](TECH-okf-knowledge-bundle.md)
- [TECH-live-wiki-resource-provider.md](TECH-live-wiki-resource-provider.md) (proposed; human review required)
- [TECH-2026-06-01-knowledgebase-backend-design.md](TECH-2026-06-01-knowledgebase-backend-design.md)
- [TECH-2026-06-01-knowledgebase-backend-phase1-implementation.md](TECH-2026-06-01-knowledgebase-backend-phase1-implementation.md)
- [TECH-2026-06-09-knowledgebase-agent-rag-design.md](TECH-2026-06-09-knowledgebase-agent-rag-design.md)
- [TECH-2026-06-09-knowledgebase-agent-rag-implementation.md](TECH-2026-06-09-knowledgebase-agent-rag-implementation.md)
- [TECH-2026-06-12-knowledgebase-open-api-design.md](TECH-2026-06-12-knowledgebase-open-api-design.md)
- [TECH-2026-06-12-knowledgebase-open-api-implementation.md](TECH-2026-06-12-knowledgebase-open-api-implementation.md)
- [TECH-topology-standard.md](TECH-topology-standard.md)
- [PRD-mvp-launch.md](../../product/prd/PRD-mvp-launch.md)
- [ADR-20260624-phase2-postgres-rls-multi-tenant.md](../decisions/ADR-20260624-phase2-postgres-rls-multi-tenant.md)
- [ADR-20260713-group-knowledgebase-binding-and-launch.md](../decisions/ADR-20260713-group-knowledgebase-binding-and-launch.md)
- [ADR-20260720-knowledge-engine-provider-binding-spi-v2.md](../decisions/ADR-20260720-knowledge-engine-provider-binding-spi-v2.md) (accepted)

## 1. Architecture Overview

SDKWork Knowledgebase is a Rust backend with a horizontally scalable application public-ingress process, a separately scalable worker process, and a PC React client (browser + optional Tauri desktop). The public ingress mounts app-api, backend-api, and open-api route surfaces on one application-plane listener. Each production deployment binds **one tenant per ingress/worker process** with fail-closed tenant and organization guards.

| Surface | Prefix | SDK family | Auth |
|---------|--------|------------|------|
| App API | `/app/v3/api` | `sdkwork-knowledgebase-app-sdk` | dual-token |
| Backend API | `/backend/v3/api` | `sdkwork-knowledgebase-backend-sdk` | dual-token + `knowledge.platform.manage` |
| Open API | `/knowledge/v3/api` | `sdkwork-knowledgebase-sdk` | API key |
| Worker | — | — | internal |

OpenAPI contracts are authored in `sdks/*/openapi/`, synchronized to `apis/` via `pnpm api:materialize`, and consumed by generated TypeScript SDKs.

## 2. Technology Choices

- **Backend**: Rust, Axum, SQLx, `sdkwork-web-framework`, PostgreSQL (production), SQLite (local dev)
- **Storage**: `sdkwork-drive` via `sdkwork-knowledgebase-drive` adapter only
- **OKF browser views**: PC file lists use `spaces.browser.list?view=files`, which resolves OKF spaces to `sources/raw` original files. OKF bundle inspection uses `view=okf_bundle`; generated outputs use `view=outputs`.
- **Memory**: `sdkwork-memory` via `sdkwork-knowledgebase-memory` port only
- **Frontend**: React 19, Vite, TipTap, IAM app SDK, generated knowledgebase app SDK, `@sdkwork/drive-app-sdk` for persistent uploads
- **Client pagination**: PC Cloud Drive browse/import uses generated Knowledgebase SDK and Drive SDK cursor page methods; interactive my-drive, starred, recent, and shared tabs load one page at a time and never prefetch multi-page aggregates
- **Client composition**: native authority per `APP_COMPOSITION_SPEC.md` — root `pnpm-workspace.yaml`, pc-core `sdkDependencies`, and capability packages import SDK types only via `sdkwork-knowledgebase-pc-core/sdk`
- **Observability**: Prometheus `/metrics` (in-cluster only), structured audit logs, optional OTLP
- **External knowledge Providers**: `kb_provider_binding` is the sole tenant/organization/space
  selection authority. `sdkwork-knowledgebase-provider-runtime` owns outbound target policy,
  deadlines, bounded retries, `Retry-After`, circuit breaking, bulkheads, response limits, trace
  propagation, redaction, and bounded-cardinality Provider metrics. Source rows never select a
  Provider. `KnowledgeEngineSpaceResolver` returns a `KnowledgeEngineExecutionHandle`; search,
  read, and list validate immutable request-derived identity, scope, binding, trace, and deadline
  before engine execution. Adapters revalidate request tenant/space before HTTP, and external sync
  requires the same explicit context. Credential-free infrastructure probes may use the bounded
  system-health context; external Binding health always carries an authenticated management
  context and resolves the Binding credential after authorization.

## 3. System Boundaries

- Business logic: `sdkwork-knowledgebase-service`
- Persistence: `sdkwork-knowledgebase-repository-sqlx` + `database/` lifecycle
- HTTP boundaries: `sdkwork-routes-knowledgebase-{app,backend,open}-api`
- Background work: `sdkwork-knowledgebase-worker` (outbox, ingest, group archive, and Provider
  migration maintenance)
- Ingestion workers atomically claim Drive jobs with owner/token leases, renew leases during processing, reclaim expired work after crashes, and fence stale workers from success or failure commits. Chunk replacement, job completion, and outbox append remain one database transaction.
- Production Snowflake generators obtain fenced node IDs from `sdkwork_node_registry`. Lease loss disables ID generation and fails runtime readiness; Kubernetes supplies only the pod UID identity, never a hashed node ID.
- Media tasks consume the generated `clawrouter-open-sdk` through the existing credential-resolving provider boundary. Image requests require URL output to keep base64 image payloads out of process memory; transcription accepts bounded HTTPS references and rejects local/private hosts.
- Wiki publication projects Drive nodes under the fixed `sources/raw` root into per-file source,
  publication, visibility, route, render, and index state. Eligible Markdown pages and static assets
  resolve live through the typed Knowledgebase Wiki provider; ordinary content changes do not build
  `kb_site_release`. Deploy owns Site/domain/Variant/TLS/runtime configuration and Web Server owns
  public HTTP/TLS/cache execution. See
  [ADR-20260721-live-mounted-wiki-publication.md](../decisions/ADR-20260721-live-mounted-wiki-publication.md)
  and [TECH-live-wiki-resource-provider.md](TECH-live-wiki-resource-provider.md).
- Backend administrative list handlers use cursor page contracts and push ordering, filtering, and limits into database queries; full-list downloads are not a production path.
- The persistence contract stores only write-only Provider credential references, never plaintext
  credentials. Binding lifecycle and Provider-to-Provider migration checkpoints are
  tenant/organization scoped, version-fenced, RLS protected, and retain predecessors for rollback.
  After execution-handle authorization, runtime resolution loads the current Binding reference
  through an injected port and passes a one-time redacted/zeroized credential to the adapter.
  Adapter startup config and route crates never read secrets. The dedicated Provider secret adapter
  requires a tenant/organization/space/Binding/actor/operation/trace/deadline access context.
  Development and test allow only namespaced
  `env://SDKWORK_KNOWLEDGEBASE_PROVIDER_SECRET_<PROVIDER_CODE>_*` values and bounded regular
  `file://` values whose canonical path remains under
  `SDKWORK_KNOWLEDGEBASE_PROVIDER_SECRETS_DIR/<provider-code>/`. Cross-Provider credential locators
  fail closed. Staging and production
  allow only `secret://knowledgebase/provider/...` through an injected Kernel `SecretProvider`;
  default production construction fails closed when no approved resolver is provided. Resolution
  requires a managed audit record identifier, caps values at 64 KiB, and applies one five-second-or-
  shorter request budget across admission and backend execution. A 32-call default bulkhead retains
  permits for timed-out synchronous calls until the backend returns, so stuck backends cannot create
  unbounded blocking work; file buffers and intermediate plaintext are zeroized on every exit path.
  Telemetry emits no locator, secret id, or value and uses fixed outcome categories. Management
  writes validate the active environment policy before persistence
  without loading secret material. No secret cache exists, so rotation/revocation cannot leave
  stale process-local credentials. The backend
  management authority and generated SDKs provide credential-reference create/list/retrieve/rotate/
  revoke, space-scoped Binding create/list/retrieve/update/test/activate/disable, and migration
  create/list/retrieve/rollback with optimistic versions and resource-aware mutation audit. The
  migration Worker uses fenced owner/token leases, one phase per claim, durable checkpoints,
  observation deferral, and transactional Binding cutover/rollback. Audit payloads are field-whitelisted to resource
  type/id, URL space, expected/result version, and result status; credential locators, fingerprints,
  remote resource IDs, raw requests, and secret values cannot enter the audit event. Provider
  Certification v2 executes a versioned six-dimension contract suite for all ten adapters and
  fingerprints its local evidence. Live promotion is a separate release gate requiring a pinned
  upstream version and SHA-256-bound quality, contract, load/SLO, outage, licensing, and
  security/privacy evidence with adapter-commit, workflow, reviewer, expiry, and approval bindings.
  Quality evidence additionally requires a reviewed production-domain dataset, bounded minimum
  coverage, exact query/run cardinality, raw result and report provenance, and deterministic metric
  recomputation; contract samples cannot satisfy this gate.
  The dedicated PC backend-admin Provider package exposes credential-reference, Binding, and
  migration workflows through admin-core and the composed backend SDK with cursor pagination,
  optimistic versions, capability/lifecycle guards, write-only locator inputs, and sanitized states.
  A separate one-shot Worker operations entrypoint produces the prelaunch active-external-space
  readiness report through a service read-model port and SQLx adapter. It requires explicit tenant
  and organization scope, uses opaque keyset pagination at the store, and cannot read sources,
  credentials, or remote Provider resource identifiers. It is informational only and cannot create
  or infer Bindings.
  Current `liveCertifiedCount` is zero. A concrete production secret-manager/KMS backend with
  bounded TLS transport, durable audit retention and rotation/revocation drill evidence,
  authenticated operator UI acceptance, release PostgreSQL evidence, and live certification remain
  prelaunch gates.
- Provider health probes native infrastructure without credentials and probes external Providers
  only through active Bindings using the authenticated backend Operator and trace. External probes
  stream paginated Bindings with a fixed concurrency bound of eight and the request deadline. The
  health path never binds an external Provider at startup or fabricates a system business context.
- PC client: `apps/sdkwork-knowledgebase-pc/`

## 4. Security Model

- Production boot is fail-closed: Postgres, Redis rate limiting, secrets encryption, web audit persistence
- Backend OpenAPI declares `x-sdkwork-permission: knowledge.platform.manage` on all protected operations
- Public ingress exposes API paths only; `/metrics` is scraped via ServiceMonitor inside the cluster
- PC production builds disable demo/mock API fallbacks
- Managed group spaces use `kb_group_knowledge_space_binding` instead of generic context binding.
  The binding is scoped by tenant, organization, and IM Conversation id; group spaces are hidden
  from generic resource routes and resolved only through the specialized launch path.
- The group resolver requires both a synchronized IM role snapshot and direct Drive authorization.
  Current-Owner initialization and active-content access are separate: only the current IM Owner
  may initialize or retry failed provisioning. Once active, Owner maps to Owner, Admin to Writer,
  Member to Reader, and Guest to no access; left, removed, and non-member actors are also denied.
  ACL projection failure is fail-closed, and `active` binding state requires an active ACL
  projection.
- IM launch tickets are opaque, hash-stored, one-time, short-lived capabilities bound to verified
  actor/session scope, binding version, and membership epoch. Browser tickets are fragment-only;
  desktop tickets are transient deep-link data and never persistent host state.

## 4.1 Managed Group Knowledgebase Boundary

IM owns the Conversation roster and lifecycle. Knowledgebase owns the one-to-one managed binding,
space/Drive lifecycle, ACL projection, and final content enforcement. Trusted IM service calls use
the generated SDK/RPC boundary; the authenticated Knowledgebase App API consumes a ticket and
resolves the exact binding target. IM alone applies current-Owner initialization and retry
authorization before it requests provisioning; Knowledgebase never treats a browser-supplied role
as authority. It accepts launch tickets only after the binding is active and the interactive caller
is a joined non-Guest Owner, Admin, or Member. The browser opens the standalone `/group-launch`
route under its configured public base path. The desktop handoff uses the independent Knowledgebase
Tauri process, not an IM-owned iframe or Webview. See
[ADR-20260713-group-knowledgebase-binding-and-launch.md](../decisions/ADR-20260713-group-knowledgebase-binding-and-launch.md).

## 5. Deployment Topology

Production uses `cloud.production`; process decomposition remains an implementation detail inside that profile. Kubernetes runs one replicated `application.public-ingress` Deployment for all application HTTP route surfaces and one replicated worker Deployment. The platform cloud gateway preserves distinct app/backend/open authorities while routing them to the same bounded public-ingress Service. See `deployments/README.md` and `etc/topology/`.

## 6. Verification

```bash
pnpm check
pnpm check:app-composition
node ../sdkwork-specs/tools/check-api-response-envelope.mjs --workspace .
pnpm verify
pnpm test
```

Gates include architecture alignment, `verify-repo` native composition, PC app hygiene (SDK boundary), utils integration, API envelope, SDK generation, database contract, and Phase 1/2 readiness scripts.

Phase 1.0 launch acceptance: [PRD-mvp-launch.md](../../product/prd/PRD-mvp-launch.md).  
Phase 2 commercial SaaS: [PRD-phase2-commercial-saas.md](../../product/prd/PRD-phase2-commercial-saas.md).
