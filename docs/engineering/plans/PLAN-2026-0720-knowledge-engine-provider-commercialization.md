# PLAN-2026-0720 Knowledge Engine Provider Commercialization

Status: active  
Requirement: REQ-2026-0720  
Decision: ADR-20260720-knowledge-engine-provider-binding-spi-v2 (accepted)  
Owner: SDKWork Knowledgebase maintainers  
Updated: 2026-07-20

## Execution Rules

- Work in evidence loops: failing test/check, narrow implementation, narrow verification, broader
  gate, then rescan. A green mock test is not live provider certification.
- `ADR-20260720` was accepted on 2026-07-20. Any implementation that changes its public naming,
  security model, credential ownership, migration direction, or release governance requires a new
  human-reviewed decision rather than a compatibility workaround.
- Preserve prelaunch publication gates until all external evidence exists.

## Phase 0: Truth And Determinism

Status: implemented, focused verification passing.

- [x] Align catalog and vendor integration tiers/categories.
- [x] Publish runtime capabilities and reject manifest/runtime drift.
- [x] Remove fake list semantics from all adapter-tier providers.
- [x] Preserve native mode and reject ambiguous external provider inference.
- [x] Probe native infrastructure directly and active external Bindings through authenticated,
  binding-scoped credential resolution; remove unbound external health and false green.
- [x] Reject duplicate registry IDs without replacement.

Exit evidence: catalog/SPI checkers; contract, resolver, registry, adapter, and hosted health tests.

## Phase 1: SPI v2 Review Gate

- [x] Record requirement, global audit, ADR, implementation plan, migration/rollback outline.
- [x] Human architecture/data/API/SDK/security/privacy review accepted the ADR on 2026-07-20.
- [x] Create the dated `MIG-*` direct-cleanup record for the unreleased application.

Exit condition: complete. Persistence and public API implementation may proceed under the accepted
decision and `MIG-2026-0720`.

## Phase 2: Shared Runtime Foundations

- [x] Introduce the stable provider error model with safe detail and retry metadata.
- [x] Implement shared HTTP client policy, deadlines, cancellation, retries, `Retry-After`, circuit
  breaker, bulkhead, body limit, trace propagation, metrics, and redaction.
- [x] Add the standard execution-context conversion and Runtime preflight validation for
  tenant/organization/actor/data-scope/space/binding/trace/deadline before scoped HTTP execution.
- [x] Convert all ten executable adapters; fail static checks on bare HTTP clients.
- [x] Add deterministic wire/error/resilience tests and concurrency tests.

Exit condition: all adapters use the shared runtime and pass fault-injection tests with no secret or
unbounded body exposure.

## Phase 3: Explicit Binding And Management Plane

- [x] Add approved binding/credential-reference/migration persistence and RLS/index contracts.
- [x] Make the active tenant/organization/space binding the sole external resolution authority and
  instantiate adapters with the binding-owned remote resource.
- [x] Define execution-context, capability, lifecycle, binding, and Provider failure contracts; use
  the context for management authorization and scope checks.
- [x] Propagate immutable request-derived execution context through search, read, list, and sync
  SPI surfaces. `KnowledgeEngineExecutionHandle` validates tenant, organization, actor, permission,
  data scope, space, binding, trace, and deadline before engine execution; the Provider Runtime
  revalidates request tenant/space before HTTP. App API and Agent Chat preserve the authenticated
  actor and trace, and adapter clients cannot fabricate business execution contexts.
- [x] Resolve write-only credential references only after execution-handle authorization. All ten
  adapters consume one-time redacted/zeroized secrets through `bind_provider`; startup config never
  reads credentials; env/file locators fail closed; no secret cache exists; aggregate health is
  Binding-aware; authorization-order and zero-secret regression tests are executable.
- [x] Implement credential-reference create/list/retrieve/rotate/revoke management operations with
  pre-persistence locator validation, optimistic versions, resource/version-fenced mutation audit,
  secret-safe read models, and immediate fail-closed revocation. Audit persistence accepts only
  resource type/id, URL space, expected/result version, and result status; missing Operators and
  persistence failures fail closed. Cache invalidation is not applicable until a reviewed bounded
  cache is introduced; the current no-cache policy takes effect on every operation.
- [x] Implement space-scoped Binding list/retrieve/create/update/test/activate/disable operations
  through authored OpenAPI and regenerated TypeScript/Rust backend SDKs.
- [ ] Inject a production secret-manager/KMS resolver and complete credential operator runbooks.
- [x] Implement Provider migrate control-plane operations through the recoverable migration worker
  and generated TypeScript/Rust backend SDKs. Provider sync remains capability-gated on each
  adapter and is not represented as a fake migration data-copy operation.
- [x] Add worker ownership, idempotency, leases, checkpoints, optimistic concurrency, atomic
  cutover/rollback, bounded batches, structured results, and transition audit.
- [x] Add the Provider management UI in the dedicated
  `sdkwork-knowledgebase-pc-admin-provider` backend-admin package. It uses the composed backend SDK
  through admin-core, server cursor pages, write-only locator inputs, optimistic versions,
  lifecycle/capability-aware Binding actions, migration rollback guards, permission denial, safe
  errors, empty/loading states, and package-owned i18n.

Exit condition: SQLite and PostgreSQL behavior, API/SDK gates, tenant/actor/data-scope isolation,
concurrency, and lifecycle recovery tests pass.

## Phase 4: Migration And Rollback

- [x] Apply the approved prelaunch direct cutover with no source resolver, dual read, dual write,
  compatibility alias, or feature flag.
- [ ] Produce a bounded prelaunch data report for external spaces that have no active binding;
  require explicit administrator creation rather than synthesizing bindings from source order.
- [ ] Pilot explicit binding by tenant, validate retrieval quality and SLOs.
- [x] Prove atomic cutover, retained predecessor, observation-window claim deferral, stale-worker
  fencing, and rollback on SQLite. Release PostgreSQL and live-provider proof remains an exit gate.
- [x] Remove source-based Provider selection in the same binding cutover.
- [x] Remove `KnowledgeSourceStore`, source metadata parsers, and legacy runtime constructors from
  every external adapter; prove Binding-owned remote-resource traffic and enforce the boundary in
  SPI/catalog static checks.

Exit condition: release-environment PostgreSQL migration, cutover, outage, rollback, reconciliation,
and backup/restore evidence is attached.

## Phase 5: Certification And Commercial Exit

- [x] Require health success/degradation, search, read, and unsupported-list contract evidence for
  all ten adapter-tier providers; keep live certification explicitly pending.
- [x] Add a versioned offline evaluation runner with Recall@K, MRR, nDCG@K, citation correctness,
  failure-rate, P95 latency, empty-query thresholds, and failure exit status.
- [x] Add the production-domain evaluation evidence contract and anti-fabrication gate. It rejects
  contract fixtures, duplicate/unknown runs, invalid schemas, negative latency, insufficient query
  and rejection coverage, fewer than two reviewers, mutable Provider versions/commits, stale
  evidence, missing or digest-mismatched dataset/results/report artifacts, and metrics that differ
  from deterministic recomputation.
- [x] Versioned Provider contract suite `1.0.0` for capability, authentication, error mapping,
  resilience, tenant/space isolation, and health. All ten executable adapters pass their complete
  owned crate suites; evidence sources are SHA-256 fingerprinted and commands execute without a
  shell. This is local contract certification only.
- [ ] Replace the contract sample with reviewed production-domain golden datasets and collect
  version-pinned results for every production-tier provider.
- [ ] Live certification matrix for every production-tier provider/upstream version.
- [ ] Licensing/redistribution, data processing/residency/retention/deletion, vulnerability, SBOM,
  provenance, load, SLO, alert, outage, rollout, rollback, and operator runbook evidence.
- [ ] Full `pnpm check`, `pnpm verify`, `pnpm test`, PostgreSQL, frontend, Playwright, API/SDK,
  security, supply-chain, and release gates on the immutable release candidate.

Exit condition: no open P0/P1 finding, every P2 has an owner and approved disposition, all automated
and external evidence is green, human reviewers sign off, and publication governance is activated.

## Continuous Rescan

After each phase, scan for capability drift, native-only health filtering, source-order selection,
duplicate registration, direct HTTP client construction, raw secrets, unbounded reads/retries,
missing scope/trace/deadline, fake success, full-list pagination, untested provider versions, and
documentation claiming more than the evidence proves.

## 2026-07-20 Credential Boundary Evidence

- `cargo test --no-run -p sdkwork-intelligence-knowledgebase-service -p sdkwork-routes-knowledgebase-backend-api -p sdkwork-routes-knowledgebase-app-api`: passed in an isolated target directory.
- Full service/backend/app package tests: passed, including service `87/87`, execution ordering
  and bounded health `6/6`, App API `24/24`, hosted routes `50/50`, and backend context/security
  tests.
- Full Dify, RAGFlow, Onyx, AnythingLLM, Open WebUI, Flowise, Chroma, Qdrant, Weaviate, and
  Haystack package matrices: passed.
- `node tools/check_knowledge_engine_spi_standard.mjs` and
  `node tools/check_external_knowledge_engine_catalog.mjs`: passed; the latter validates 12 catalog
  vendors and all executable adapter credential boundaries.
- SDKWork response envelope, pagination, SDK ownership/consumer imports, application layering, Rust
  composition, component port, and identity naming checks: passed. Root `pnpm check` is green and
  API/route-manifest materialization is idempotent.
- `check-api-operation-patterns.mjs` remains red because
  `POST /backend/v3/api/knowledge/okf/index/rebuild` is still exposed as `create` with `201` instead
  of the standard `rebuild` command with `200` or async `202`; public naming review is required
  before changing the authored route, OpenAPI, and generated SDK surface.
- `pnpm test` reaches the topology suite but remains red because
  `deployments/docker/Dockerfile.api` still builds the retired standalone-gateway crate/binary name;
  changing production deployment config requires human review. Frontend, security, observability,
  launch-runbook, E2E guard, database-contract, and SDK-ownership suites pass independently.
- Live browser/open/backend API smoke tests were skipped because no release-environment URLs and
  credentials were configured. Evidence scope remains implementation/contract only and does not
  satisfy live Provider certification, production PostgreSQL, migration/rollback, load/SLO,
  supply-chain, rollout/rollback, or release gates.

## 2026-07-20 Provider Management Plane Evidence

- The backend authority contains 16 Provider management operations: five credential-reference,
  seven space-scoped Binding, and four space-scoped migration operations. Every operation uses dual-token
  `WebRequestContext`, `knowledge.platform.manage`, standard SDKWork v3 envelopes, numeric
  `ProblemDetail` errors, mutation audit, and cursor pagination where applicable.
- Credential create/rotate validates `env://UPPERCASE_VARIABLE` or syntactically valid `file://`
  locators before persistence without loading a secret. Read/list responses have no locator or
  fingerprint field. Rotation and revocation are version-fenced; revoked references cannot be
  resolved or rotated.
- `cargo test -p sdkwork-intelligence-knowledgebase-repository-sqlx --test provider_binding_store`
  passes `3/3`, covering tenant/organization isolation, store-level pagination, stale versions,
  rotation, revocation, and fail-closed resolution.
- The targeted Hosted runtime management test passes the real SQLite route/service/store stack for
  credential create/list/rotate/revoke and Binding create/update, wrong-space denial, stale-version
  conflict, string int64 wire fields, command envelopes, invalid locator rejection, and secret
  non-disclosure.
- TypeScript and Rust backend SDK generation was applied with reviewed generator fingerprints. A
  second dry-run reports `hasChanges=false` and `hasDestructiveChanges=false` for both. Credential
  Binding, and migration list methods return named Page models instead of untyped records/JSON values.
  Generated TypeScript build and generated Rust `cargo check` plus `cargo test --no-run` pass. The
  generator's npm-based TypeScript publish helper
  remains unusable in this pnpm workspace because npm attempts to rebuild the pnpm-store Rollup
  package without Rollup's repository-only build dependencies; repository-native pnpm build is the
  accepted code verification for this round, not release publication evidence.
- `api:materialize:check`, SDK ownership, app SDK consumer imports, API response envelope, and
  pagination gates pass. Operation-pattern validation has no Provider management violation; its
  only remaining failure is the separately reviewed OKF rebuild public contract.
- Resource-aware Provider audit persistence tests pass `20/20` in the observability crate. Full
  service, SQLx repository, backend route, and app route suites pass. Security, observability,
  launch-readiness, Phase 2 readiness, frontend lint, and backend
  admin SDK-consumer tests pass independently.
- API assembly materialization is idempotent, the standalone gateway consumes the assembly through
  `ApiAssembly::from_environment`, structure verification passes, and the canonical gateway compiles.
  The legacy Phase 1 verifier is now read-only: it uses materialization/SDK/assembly checks and never
  deletes local generated SDK artifacts. Root `pnpm verify` currently stops on ignored local `dist/`
  build output until an approved clean is run; this is not release publication evidence.

## 2026-07-20 Provider Migration Plane Evidence

- The migration store implements idempotent creation, one active migration per space, bounded
  cursor pagination, expiring owner/token leases, stale-claim fencing, optimistic versions,
  checkpointed phase transitions, observation-window claim deferral, and terminal failure state.
- Cutover and rollback update the operation plus both Bindings in one database transaction. The
  predecessor Binding and both remote resources are retained; no worker path deletes or fabricates
  Provider data. `pre_provisioned_target` is the explicit preparation mode until a separately
  reviewed data-transfer capability exists.
- The Worker processes one durable phase per claim and reports processed, completed, rolled-back,
  and failed counts. `SDKWORK_KNOWLEDGEBASE_WORKER_PROVIDER_MIGRATION_BATCH_SIZE` and
  `SDKWORK_KNOWLEDGEBASE_WORKER_PROVIDER_MIGRATION_LEASE_SECONDS` are independent operational
  controls. Every phase emits a durable, field-whitelisted system audit event.
- Backend routes provide create/list/retrieve/rollback under the exact URL space scope. Public DTOs
  expose decimal-string int64 fields and never expose checkpoint, claim owner/token, or lease data.
  The TypeScript and Rust generator plans contained no destructive changes; repeated dry-runs are
  idempotent, and both generated packages build/check successfully.
- Repository tests cover lifecycle, idempotency, exclusivity, lease recovery, failed-state rollback,
  atomic cutover/rollback, and observation deferral. The Hosted runtime test covers the real auth,
  route, service, Worker, SQLx, envelope, pagination, wrong-space hiding, cutover, and rollback path.
- This evidence is implementation-level SQLite proof, not release approval. Production PostgreSQL,
  live Provider quality/SLO, backup/restore, outage simulation, load, security/privacy, supply-chain,
  immutable artifact, and operator sign-off remain mandatory.

## 2026-07-20 Provider Certification v2 Evidence

- `provider-certification.manifest.json` schema v2 separates local `contractCertification` from
  `liveCertification`. Ten executable Providers are locally `passed`; LangChain and LlamaIndex are
  non-executable catalog entries and remain `not-applicable`; `liveCertifiedCount` is exactly zero.
- Contract suite `1.0.0` requires capability, authentication, error mapping, resilience, isolation,
  and health evidence. Every evidence path must exist, its combined SHA-256 fingerprint must match,
  and each structured `cargo test -p <owned-adapter> --all-targets` command is shell-free and
  injection-checked. All ten adapter commands passed across the recorded executions.
- Live promotion requires a pinned upstream version and current, immutable release evidence under
  `docs/releases/provider-certification/`. The evidence index and each quality, contract, load/SLO,
  outage-recovery, licensing, and security/privacy artifact carry SHA-256 digests. The schema also
  binds the adapter commit, workflow run, reviewer, expiry, and approved legal/security gates.
- The checked-in template is deliberately `draft` with a template-only kind and pending approvals;
  a regression test proves it cannot be accepted as certified evidence. These controls prevent
  local mock results or placeholder documents from manufacturing production status.
- The quality-evaluation evidence schema requires at least 50 scored production-domain queries,
  three rejection cases, two distinct reviewers, a pinned upstream version and adapter commit, and
  digest-bound dataset, raw results, and evaluation report files. The gate validates exact
  one-to-one query runs and deterministically recomputes all metrics. Its positive test uses only an
  ephemeral test tree; no production golden dataset or passing Provider result is checked in.

## 2026-07-20 Provider Management UI Evidence

- `/admin/providers` is lazy-loaded from the normalized private backend-admin capability package,
  not from an app/user feature. The existing `/admin` overview links to it and remains unchanged.
- Credential reference list/create/rotate/revoke, space Binding list/create/update/test/activate/
  disable, and migration list/create/rollback call only the composed Knowledgebase backend SDK via
  admin-core. No raw HTTP, manual authorization header, local DTO fork, or generated transport import
  exists in the feature package.
- Credential locators use password inputs with autocomplete disabled and appear only in create/
  rotate requests. Returned rows contain only the secret-safe API model. Commands submit the current
  decimal-string version; Binding activation is enabled only after tested `health` and `search`
  capabilities, and rollback is enabled only for backend-supported states.
- Three operational tables request 20-row cursor pages and maintain bounded previous/next history.
  Six Vitest tests cover action rules, cursor/version SDK calls, permission denial, loading, empty,
  sanitized error, and successful capability-aware rendering. PC TypeScript, app composition,
  SDK-consumer, hygiene, shell/admin-core tests, and production Vite build pass; the isolated Provider
  feature chunk is 25.99 kB (8.26 kB gzip).
- Browser routing redirects unauthenticated access to
  `/auth/login?redirect=/admin/providers`, proving the admin route is protected. A real local admin
  visual acceptance run is blocked by existing IAM PostgreSQL drift: standalone bootstrap attempts
  to recreate `iam_session_service_account_credential_fk` and fails closed. No migration was changed
  or bypassed; release-environment operator/visual acceptance remains mandatory.
