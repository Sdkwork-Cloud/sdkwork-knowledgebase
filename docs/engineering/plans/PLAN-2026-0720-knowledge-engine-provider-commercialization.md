# PLAN-2026-0720 Knowledge Engine Provider Commercialization

Status: active  
Requirement: REQ-2026-0720  
Decision: ADR-20260720-knowledge-engine-provider-binding-spi-v2 (accepted)  
Owner: SDKWork Knowledgebase maintainers  
Updated: 2026-07-20

## Execution Rules

- Work in evidence loops: failing test/check, narrow implementation, narrow verification, broader
  gate, then rescan. A green mock test is not live provider certification.
- Stop before schema, public API/SDK naming, auth/security semantics, credential ownership, generated
  SDK ownership, or release/deployment changes until the proposed ADR receives human acceptance.
- Preserve prelaunch publication gates until all external evidence exists.

## Phase 0: Truth And Determinism

Status: implemented, focused verification passing.

- [x] Align catalog and vendor integration tiers/categories.
- [x] Publish runtime capabilities and reject manifest/runtime drift.
- [x] Remove fake list semantics from all adapter-tier providers.
- [x] Preserve native mode and reject ambiguous external provider inference.
- [x] Include configured external providers in aggregate health and remove false green.
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
- [ ] Propagate the immutable execution context through every search/read Provider call and require
  tenant/space/binding/data-scope/deadline validation before credential resolution or HTTP.
- [ ] Implement backend list/retrieve/create/update/test/activate/disable/sync/migrate operations
  through authored OpenAPI and regenerated composed SDKs.
- [ ] Add worker ownership, idempotency, leases, checkpoints, optimistic concurrency, and audit.
- [ ] Add the provider management UI with capability-aware actions and sanitized status.

Exit condition: SQLite and PostgreSQL behavior, API/SDK gates, tenant/actor/data-scope isolation,
concurrency, and lifecycle recovery tests pass.

## Phase 4: Migration And Rollback

- [x] Apply the approved prelaunch direct cutover with no source resolver, dual read, dual write,
  compatibility alias, or feature flag.
- [ ] Produce a bounded prelaunch data report for external spaces that have no active binding;
  require explicit administrator creation rather than synthesizing bindings from source order.
- [ ] Pilot explicit binding by tenant, validate retrieval quality and SLOs.
- [ ] Prove atomic cutover, retained predecessor, observation window, and rollback.
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
- [ ] Versioned provider contract suite for capability, auth, error, resilience, isolation, and health.
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
