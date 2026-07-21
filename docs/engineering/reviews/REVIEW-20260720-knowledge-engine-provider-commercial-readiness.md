# REVIEW-20260720 Knowledge Engine Provider Commercial Readiness

Status: open  
Requirement: REQ-2026-0720  
Owner: SDKWork Knowledgebase maintainers  
Reviewed: 2026-07-21
Scope: catalog, SPI, runtime resolution, health, adapters, security, resilience, observability,
quality, management, migration, certification, and release evidence

## Executive Decision

The repository now has a production-oriented multi-provider implementation baseline: deterministic
catalog/capability truth, explicit Binding authority, immutable execution context, credential
references, shared resilience, management API/SDK, durable migration/rollback, sanitized audit and
offline evaluation are implemented and locally verified. Current status remains
**prelaunch-gated / certification-pending** because a concrete production secret backend and its
operational evidence, operator browser acceptance, release PostgreSQL, live Provider/version
certification, load/SLO, licensing, privacy, supply-chain,
backup/restore, rollout, rollback, immutable artifact evidence, and human release approval are not
facts that local code can supply.

## Maturity Scorecard

| Dimension | Current | Commercial target | Primary gap |
| --- | ---: | ---: | --- |
| Catalog and capability truth | 4/5 | 5/5 | Live version-pinned certification evidence |
| Deterministic selection | 5/5 | 5/5 | Maintain active-Binding-only authority checks |
| SPI semantic completeness | 4/5 | 5/5 | Live optional-capability certification |
| Tenant/security isolation | 4/5 | 5/5 | Production secret-manager/KMS and PostgreSQL RLS proof |
| Resilience | 4/5 | 5/5 | Release load/outage/SLO evidence |
| Observability | 4/5 | 5/5 | Production dashboards, alerts, and retention proof |
| Management lifecycle | 4/5 | 5/5 | Release-environment operator/browser acceptance |
| Migration and rollback | 4/5 | 5/5 | Release PostgreSQL/live-provider/backup-restore proof |
| Retrieval quality | 3/5 | 5/5 | Reviewed production-domain datasets and live results |
| Provider certification | 2/5 | 5/5 | Live version matrix, licensing and SLO evidence |

## Provider Portfolio Truth

| Provider | Category | Tier | Proven executable surface | Commercial status |
| --- | --- | --- | --- | --- |
| Dify | knowledge platform | adapter | health, search, read | live certification pending |
| RAGFlow | knowledge platform | adapter | health, search, read | live certification pending |
| Onyx | knowledge platform | adapter | health, search, read | live certification pending |
| AnythingLLM | knowledge platform | adapter | health, search, read | live certification pending |
| Open WebUI | knowledge platform | adapter | health, search, read | live certification pending |
| Flowise | orchestration/retrieval | adapter | health, search, read | live certification pending |
| Haystack | pipeline runtime | adapter | health, search, read | live certification pending |
| Chroma | retrieval infrastructure | adapter | health, search, read | not a document-management provider |
| Qdrant | retrieval infrastructure | adapter | health, search, read | not a document-management provider |
| Weaviate | retrieval infrastructure | adapter | health, search, read | not a document-management provider |
| LangChain | framework catalog | catalog | none | discovery only |
| LlamaIndex | framework catalog | catalog | none | discovery only |

`adapter` means owned executable code and deterministic contract tests. It does not mean production
certification, supported ingest/sync, upstream-version compatibility, licensing approval, or SLO.

## Closed Findings

- P0 catalog root/vendor tier and category drift is rejected by tooling.
- Runtime capability descriptors match current manifest SPI mappings.
- Catalog-only frameworks expose no executable capability.
- Infrastructure providers no longer return collection/class/pipeline descriptors as documents.
- Explicit request mode wins; native space mode is not overridden by connector sources.
- External resolution requires one executable provider and rejects ambiguity.
- Configured external providers participate in aggregate health; failures degrade status.
- Duplicate registration is rejected without overwriting the original engine.
- All ten adapter-tier providers prove healthy and failed-upstream health mapping, and the catalog
  gate rejects missing health/search/read/unsupported-list contract evidence.
- A deterministic offline evaluator now defines Recall@K, MRR, nDCG@K, citation correctness,
  failure-rate, P95 latency, and empty-query gates; reviewed production datasets/results remain open.
- The production evaluation evidence contract rejects sample fixtures and requires at least 50
  and at most 5,000 scored questions, 3-500 rejection cases, two reviewers, exact one-to-one
  Provider runs, pinned versions/commits, release provenance, current non-future evidence, and
  digest-bound dataset/results/report artifacts no larger than 32 MiB. It recomputes metrics before
  acceptance; no real production dataset is attached yet.
- Active tenant/organization/space Binding is the sole external selection authority; source-order
  inference and adapter startup credential selection are absent.
- SPI execution handles authorize tenant, organization, actor, permission/data scope, space,
  Binding, trace, and deadline before credential resolution or Provider network access.
- All executable adapters use the shared Provider Runtime for deadlines, retries, Retry-After,
  circuit breaking, bulkheads, response bounds, trace propagation, metrics, and redaction.
- Credential references, Binding lifecycle, backend management API, resource/version audit, and
  generated TypeScript/Rust backend SDKs are implemented with secret-safe read models.
- Provider credentials are resolved by a dedicated backend-provider adapter rather than the route
  crate. Its typed access context carries tenant, organization, space, Binding,
  credential-reference version, implementation, actor, operation, trace and deadline. Local
  development/test sources are implementation namespace/per-Provider-root confined with canonical
  cross-Provider and symlink-escape rejection;
  staging/production require managed `secret://` injection and default startup fails closed. Values
  and total wait time are bounded, a managed concurrency bulkhead retains permits for timed-out
  synchronous calls until backend return, and file buffers/intermediate plaintext are zeroized on
  all exits. Errors and fixed-outcome telemetry are sanitized, a managed audit record identifier is
  mandatory, and tests
  prove saturation containment plus immediate rotation/revocation with no cache.
- Provider migration uses idempotent operations, fenced leases, checkpoints, observation deferral,
  atomic Binding cutover/rollback, retained predecessors, sanitized system audit, and Worker metrics.
- Provider certification manifest v2 and contract suite `1.0.0` cover capability, authentication,
  error mapping, resilience, isolation, and health for all ten executable adapters. Evidence source
  fingerprints and shell-free complete-crate commands are enforced; all ten suites pass.
- Live evidence has a versioned schema and anti-fabrication gate: a pinned Provider version, adapter
  commit, release workflow, reviewer, expiry, approved licensing/security reviews, and six existing
  SHA-256-bound release artifacts are required. The repository currently reports
  `liveCertifiedCount = 0`; the draft template cannot pass the gate.
- Load/SLO and outage-recovery evidence now have versioned schemas and a shared operational policy.
  The live gate reads digest-bound raw request samples and scenario timelines, rejects unknown or
  secret-bearing fields, escaped artifact paths, oversized artifacts/sample sets, future or
  mismatched evidence dates, and policy weakening. It recomputes aggregate/per-operation
  performance, detection, and recovery results. Test fixtures prove both passing and rejection
  behavior, but do not count as real release-environment evidence.
- A dedicated `pc-admin-provider` UI implements credential, Binding, and migration workflows through
  admin-core and the composed backend SDK. Actions are version-fenced and lifecycle/capability-aware;
  locator inputs are write-only; all lists are cursor-paged; permission/loading/empty/safe-error/
  success component states and the production build pass.
- The prelaunch Binding readiness report is implemented as a read-only service port, SQLx read
  model, and dedicated Worker command. It lists only active external spaces missing an active
  Binding, applies tenant and organization predicates in SQL, uses bounded opaque keyset pages,
  reports non-active Binding counts, and contains no source, credential, or remote-resource data.
  SQLite behavior tests pass; the optional PostgreSQL dialect probe performs no writes.

## Open Findings

| Priority | Finding | Risk | Required closure |
| --- | --- | --- | --- |
| P1 | No live provider/version certification | mocks do not prove upstream compatibility | certification matrix and release gate |
| P1 | Concrete production secret backend and drill evidence are missing | the managed injection boundary does not prove Vault/KMS TLS, timeouts, custody, audit retention or operations | approved backend integration, least-privilege policy, TLS/timeout proof, audit retention, rotation/revocation and outage drills |
| P1 | Source configuration standard is not yet enforceable | `etc/` discovery/profile authority is absent while retired `configs/` and concrete app-manifest environment URLs remain | human-reviewed production configuration migration to `etc/`, manifest cleanup, deployment validation and rollback evidence |
| P1 | Release PostgreSQL and migration evidence missing | SQLite cannot prove production locking/RLS behavior | PostgreSQL concurrency, RLS, cutover, rollback, backup/restore evidence |
| P1 | Release Provider UI acceptance not executed | local IAM database drift prevents an authenticated browser session | repair/review IAM PostgreSQL drift, then run accessibility and operator E2E acceptance |
| P1 | Real release load/outage/SLO evidence missing | commercial capacity and recovery are unproven despite the implemented evidence contract | execute version-pinned load and fault-injection runs, then attach raw samples, dashboards, alerts, and reviewed evidence |
| P1 | Supply-chain/release evidence missing | artifacts cannot be commercially published | SBOM, provenance, signature, vulnerability and immutable RC gates |
| P2 | Production-domain evaluation datasets pending | offline sample thresholds are not business acceptance | reviewed golden datasets and per-provider results |
| P2 | Licensing/residency/retention reviews pending | Provider use may violate commercial/data obligations | signed legal, privacy and data-processing matrix |

## Evidence Reviewed

- Machine catalog and all vendor manifests.
- Core contract, service registry/resolver, runtime wiring, provider health route, and ten adapter
  crates including HTTP mock tests.
- Local SPI specification, PRD, technical architecture, security/privacy/observability/performance,
  API/SDK/database/migration/test, and release standards.
- Focused passing evidence: catalog/SPI checkers; contract and execution-handle tests; all ten
  adapter HTTP/resilience suites; Provider secret-adapter namespace, canonical-path, symlink,
  context, size/time/concurrency bounds, sanitization, plaintext cleanup and no-cache
  rotation/revocation tests; Binding/credential/migration
  SQLx tests; Hosted auth/API/Worker
  cutover/rollback test; backend OpenAPI route tests; TypeScript/Rust generator zero-drift and
  language builds; response-envelope, pagination, SDK ownership and consumer-import gates; Provider
  Certification v2 unit, schema, fingerprint, catalog, and complete adapter-crate execution gates.
- Provider Binding readiness evidence: SQLx SQLite scope/filter/lifecycle/keyset tests, Worker
  command argument bounds, optional read-only PostgreSQL dialect execution, and the operator
  procedure in `docs/runbooks/RUNBOOK-provider-binding-readiness.md`.
- Provider operational-evidence tests: eight passing certification tests cover valid recomputation,
  threshold/isolation failures, all required outage scenarios, fail-open/no-alert/no-trace/retry-
  storm/secret-leak rejection, and template rejection. No live operational artifact is checked in.
- Shared certification-artifact tests and quality evidence tests prove bounded files/query sets,
  non-future evidence, deterministic recomputation, and fail-closed path/digest handling. The
  Provider certification suite passes `8/8`; the SPI/evaluation suite passes `9/9`.
- Shared SPI isolation evidence changed with the credential boundary, so all ten contract
  certification source fingerprints were recomputed using the canonical sorted-path SHA-256
  algorithm and reverified on 2026-07-21. The ten complete adapter `cargo test --all-targets`
  commands pass; `liveCertifiedCount` remains zero.
- The locked Provider Binding readiness Worker check and strict all-target Clippy for all ten
  executable Provider crates pass. Provider secret-adapter all-target Clippy, SPI, component-port
  binding, application layering, Rust backend composition, documentation, pagination, and diff
  hygiene pass. Locked route test-target compilation reaches the affected dependency graph, then
  stops in a concurrent site-publication test that still references removed `paths::SPACE_SITE`.
- Root `pnpm check` currently stops in API materialization because concurrent site-publication route
  changes have not regenerated the app-api route-manifest artifact. The Provider checks pass
  independently, and this review does not mutate an unaccepted public API surface. The dedicated source-
  config validator also reports missing `etc/README.md` and deployment profile authority, retired
  `configs/`, and concrete environment URLs in the app manifest. Production config migration
  requires human review and was not changed under ADR-20260720.
- Repository-wide verification is not green. The locked Rust route test-target check currently
  fails because `tests/integration_commerce_routes.rs` references removed `paths::SPACE_SITE`. The
  dedicated security suite separately passes `28/29`; its remaining assertion reads the concurrently
  removed `hosted_upload.rs`. Those public API/security changes require their own accepted decision
  and are not authorized by ADR-20260720.

## Commercial Blockers And Required Review

ADR-20260720 is accepted and its local implementation baseline is complete. Commercial completion
still cannot be claimed until real Provider certification and release-environment PostgreSQL/load/
outage/migration/rollback/backup evidence is attached, a concrete production secret backend and its
operational evidence plus operator UI acceptance are
accepted, licensing/security/privacy/supply-chain review is approved, and application publication
gates are deliberately activated through release governance. Local code changes cannot manufacture
those external facts.
