# REVIEW-20260721 Live Wiki Deployment Integration Readiness

Status: implementation-in-progress-delivery-blocked
Owner: SDKWork Knowledgebase maintainers
Date: 2026-07-21
Requirement: REQ-2026-0721
Decision: ADR-20260721-live-mounted-wiki-publication (accepted)
Machine contract: `specs/live-wiki-publication.spec.json`
Specs: REQUIREMENTS_SPEC.md, ARCHITECTURE_DECISION_SPEC.md, API_SPEC.md, INTERNAL_API_SPEC.md,
SDK_SPEC.md, EVENT_SPEC.md, DATABASE_SPEC.md, DEPLOYMENT_SPEC.md, RELEASE_SPEC.md,
SECURITY_SPEC.md, PERFORMANCE_SPEC.md, PAGINATION_SPEC.md, TEST_SPEC.md

## 1. Scope And Review States

This review records executable implementation evidence across Knowledgebase, Drive, Deployments,
and Web Server. It supersedes earlier absence findings that were closed by the canonical Wiki
schema, Drive event projection, Knowledgebase public provider, generated Internal SDK, and Web
Server immutable runtime-set work.

The states in this review mean:

- `closed`: the owned contract and focused executable evidence exist;
- `partially closed`: a bounded production-shaped implementation exists but a named capability or
  cross-repository proof is still missing;
- `blocking`: the required business or runtime path is absent, so public/commercial claims remain
  prohibited.

The reviewed request path is:

```text
Drive sources/raw commit
  -> Knowledgebase durable event projection
  -> WikiPublication and page public-version state
  -> Knowledgebase typed Internal API / generated SDK
  -> Web Server KNOWLEDGEBASE_WIKI adapter
  -> Deploy-owned Site/Binding/Variant/Mount routing
  -> public HTTP response
```

Ordinary source updates, publish/unpublish actions, navigation changes, and search changes are
provider lifecycle operations. They must not create a Deploy Release, Deployment, or SiteRevision.

## 2. Verdict

The storage, source-projection, explicit publication lifecycle, public-read, contract-generation,
immutable Web runtime foundations, generated-SDK Web Server provider adapter, data-plane bootstrap,
and public HTTP mapping are implemented. The repositories still do not provide the complete
integrated public Wiki product because durable Web provider-event consumption, provider-aware cache
invalidation, rendition pipeline, managed TLS closure, UI workflows, and real deployed
end-to-end production evidence remain incomplete.

The system must therefore be described as `implementation-in-progress-delivery-blocked`. It is
incorrect to describe the Wiki schema, Drive consumer, Knowledgebase provider API/SDK, or Web
runtime-set as absent. It is also incorrect to describe the overall capability as production-ready,
commercially ready, or fully realtime.

## 3. Current Evidence Matrix

| Surface | Executable evidence | State |
| --- | --- | --- |
| Canonical contract | Accepted requirement, ADR, and `specs/live-wiki-publication.spec.json` | closed |
| Wiki persistence | PostgreSQL and SQLite baselines/migrations contain `kb_site_publication`, `kb_source_file_projection`, rendition, redirect, checkpoint, inbox, and outbox structures | closed |
| Wiki initialization | One canonical DRAFT/private publication is provisioned and existing spaces are backfilled idempotently | closed |
| Drive source sync | Root-scoped events, inbox/checkpoint fencing, projection application, reconciliation, and standalone/cloud typed Drive adapters exist | closed |
| Knowledgebase public provider | Active-publication lookup, normalized route/redirect resolution, opaque content handles, exact public-version validation, navigation, and metadata search are implemented | closed |
| Internal API and SDK | Six ingress-token owner operations exist in OpenAPI, route manifest, Rust routes, and generated TypeScript/Rust transports | closed |
| Public isolation | Provider reads derive tenant/organization from the authenticated principal and use non-disclosing not-found behavior | closed |
| Web runtime descriptor/set | Strict descriptor plus node-scoped `sdkwork.website-runtime-set.v1`, bounded compilation, collision rejection, atomic activation, replay fencing, and rollback exist | closed |
| Web delivery executor | Immutable provider registry and runtime-set-backed STATIC/explicit SPA fallback/WIKI execution preserve compiled tenant/Site/Binding/Variant/Mount scope, typed outcomes, bounded streams, and browser HTTP semantics | closed |
| Content open | Exact pinned Drive version, length, SHA-256, and current page public-version are revalidated; the reader buffers at most 16 MiB and has no Range contract | partially closed |
| Search | Store-paginated metadata search covers title, canonical route, and source path; rendition-backed full-text search is absent | partially closed |
| Publication lifecycle | Owner-only activate/pause plus Writer publish/republish/unpublish/visibility commands use optimistic publication/page fences, exact Drive-version pinning, transactional lifecycle events, and transactionally coupled audit records | closed |
| App API and SDK | Six owner operations exist in App OpenAPI, Rust routes/manifest, and the generated TypeScript App SDK; Reader/Writer/Owner and organization-isolation tests pass | closed |
| Provider event production | The owner AsyncAPI authority defines all five event types; provider, route change/revocation, navigation, and search events are transactionally produced, and source-driven public revocation advances navigation/search generations and emits all three invalidation facts atomically | closed |
| Provider event consumption | Durable Web Server checkpoints, duplicate/order/gap fencing, reconciliation, and route-scoped invalidation are absent | blocking |
| Web Server Wiki adapter | Generated Knowledgebase Rust Internal SDK adapter implements resource/Wiki ports with tenant-bound resolution, conditional metadata, bounded content, navigation/search, registry/bootstrap wiring, initial/hot-update validation, and browser-facing tests | closed |
| Render/rendition safety | The target processor/sanitizer/rendition policy is documented, but the complete multi-format production chain is not executable | blocking |
| Deploy-to-Web delivery | Control-plane and data-plane contracts exist, but an activated Site-to-Wiki end-to-end delivery test is absent | blocking |
| Managed TLS | Domain/certificate policy foundations exist; automated ACME renewal, rotation, fleet convergence, and expiry-drill evidence remain incomplete | blocking |
| User/admin workflows | Generated-SDK-backed publication, source-state, domain/TLS, provider-health, reconciliation, and failure-management views are incomplete | blocking |
| Commercial launch | Release, security, performance, soak, backup/restore, billing reconciliation, rollout, rollback, and live-smoke evidence are incomplete | blocking |

## 4. Implemented Public Provider Contract

The Knowledgebase Internal API authority owns exactly these operations:

| Operation id | Method and path | Implemented behavior |
| --- | --- | --- |
| `driveEvents.receive` | `POST /internal/v3/api/knowledgebase/drive_events` | authenticated Drive event ingestion |
| `wikiPublications.retrieve` | `GET /internal/v3/api/knowledgebase/wiki_publications/{publicationUuid}` | active publication metadata and provider generations |
| `wikiPublications.routes.resolve` | `POST /internal/v3/api/knowledgebase/wiki_publications/{publicationUuid}/routes/resolve` | normalized route or reviewed redirect resolution |
| `wikiPublications.contents.retrieve` | `GET /internal/v3/api/knowledgebase/wiki_publications/{publicationUuid}/contents/{contentHandle}` | bounded exact pinned-version binary retrieval |
| `wikiPublications.navigation.list` | `GET /internal/v3/api/knowledgebase/wiki_publications/{publicationUuid}/navigation` | public-only keyset navigation window |
| `wikiPublications.pages.search` | `GET /internal/v3/api/knowledgebase/wiki_publications/{publicationUuid}/pages/search` | public-only keyset metadata search |

Direct route resolution permits `PUBLIC` and `UNLISTED`. Navigation permits only `PUBLIC` with
`nav_hidden=false`. Search permits only `PUBLIC` with `index_state=READY`. Every public read
revalidates tenant, organization, publication status, page eligibility, and current page public
version.

The current binary operation is deliberately not described as streaming: it rejects a source
representation larger than 16 MiB and returns a bounded buffered body. The current search operation
is deliberately not described as full-text: it searches normalized metadata only.

## 5. Remaining P0 Closure Work

### P0-1 Durable Provider Event Consumption

Add Web Server checkpoints, duplicate/order/gap fencing, reconciliation, and route-scoped
invalidation for `knowledgebase.wiki.provider.changed.v1`,
`knowledgebase.wiki.route.changed.v1`, `knowledgebase.wiki.route.revoked.v1`,
`knowledgebase.wiki.navigation.changed.v1`, and `knowledgebase.wiki.search.changed.v1`.
Deployments must remain outside this content hot path.

### P0-2 Rendition, Range, And Search Completion

Add a streaming/Range contract before enabling large PDF/media/download workloads. Complete the
versioned Markdown/HTML sanitizer and isolated multi-format rendition chain. Replace metadata-only
search with a tenant/publication/public-version-filtered rendition index before claiming full-text
Wiki search.

### P0-3 Integrated Delivery, TLS, And Product Operations

Prove Site/Binding/Variant/Mount-to-provider execution in standalone and cloud topologies. Complete
automatic ACME renewal/rotation and served-SNI convergence. Deliver generated-SDK-backed user/admin
workflows, provider health, lag/gap, reconcile, cache purge, quota, audit, and commercial usage
operations.

## 6. Realtime Claim Boundary

Drive-to-Knowledgebase projection and explicit public-state transitions are event-driven and
durable. Public Wiki freshness is not yet an end-to-end realtime capability because durable Web
Server event consumption, gap recovery, and provider-aware cache invalidation are not closed.

When those paths are implemented, realtime means bounded eventual visibility from the committed
public-state transition, not from upload completion. Events improve freshness; authenticated
provider read-through validation remains the correctness authority. Private, quarantine, delete,
pause, and unpublish transitions must deny public reads immediately even during event or cache lag.

## 7. Verification Evidence

The implemented provider boundary has passed:

```text
cargo test -p sdkwork-intelligence-knowledgebase-service --test wiki_public_provider
cargo test -p sdkwork-intelligence-knowledgebase-repository-sqlx --test wiki_public_provider_store
cargo test -p sdkwork-intelligence-knowledgebase-repository-sqlx --test wiki_publication_lifecycle_store
cargo test -p sdkwork-routes-knowledgebase-app-api --test wiki_publication_routes
cargo test -p sdkwork-routes-knowledgebase-app-api --test wiki_publication_hosted_access
cargo test -p sdkwork-routes-knowledgebase-app-api --test app_openapi_routes
cargo test -p sdkwork-routes-knowledgebase-internal-api --test internal_routes
pnpm api:materialize:check
node tools/knowledgebase_sdk_generate.mjs --check --family sdkwork-knowledgebase-app-sdk
pnpm --dir sdks/sdkwork-knowledgebase-app-sdk/sdkwork-knowledgebase-app-sdk-typescript typecheck
node sdks/sdkwork-knowledgebase-internal-sdk/bin/generate-sdk.mjs --check
node --test sdks/sdkwork-knowledgebase-internal-sdk/tests/sdk-family-smoke.test.mjs
pnpm --dir sdks/sdkwork-knowledgebase-internal-sdk/sdkwork-knowledgebase-internal-sdk-typescript typecheck
node ../sdkwork-specs/tools/check-api-operation-patterns.mjs --root .
node ../sdkwork-specs/tools/check-api-response-envelope.mjs --root .
node ../sdkwork-specs/tools/check-route-path-collisions.mjs --root .
node ../sdkwork-specs/tools/check-pagination.mjs --workspace .
```

The Web Server repository adapter boundary has additionally passed:

```text
cargo test -p sdkwork-webserver-contract
cargo test -p sdkwork-webserver-delivery-runtime
cargo test -p sdkwork-webserver-knowledgebase-provider
cargo test -p sdkwork-api-web-server-standalone-gateway
cargo check --workspace
cargo clippy -p sdkwork-webserver-core -p sdkwork-webserver-contract -p sdkwork-webserver-drive-provider -p sdkwork-webserver-knowledgebase-provider -p sdkwork-webserver-delivery-runtime -p sdkwork-api-web-server-standalone-gateway --all-targets -- -D warnings
```

The generated TypeScript and Rust package check/build workflows also pass. These checks prove the
bounded provider, explicit publication-command, generated-SDK Web adapter, runtime-set activation,
bootstrap, and browser HTTP mapping boundaries. They do not prove provider-event consumption,
provider-aware caching, TLS, UI, renderer, or real deployed cross-repository delivery paths.

## 8. Claim Policy

Until every remaining P0 item is closed with executable evidence:

- Wiki public deployment remains implementation-only and delivery-gated;
- upload or processing success must not be presented as public publication success;
- the current binary reader must not be presented as large-object streaming or Range delivery;
- the current metadata query must not be presented as full-text search;
- no commercial or production launch may rely on the incomplete end-to-end path.
