# REQ-2026-0720 Knowledge Engine Provider Commercialization

```yaml
id: REQ-2026-0720
title: Commercial-grade pluggable knowledge engine providers
owner: SDKWork Knowledgebase maintainers
status: in-progress
source: reliability
problem: Provider catalog entries, runtime capabilities, selection, credentials, resilience, observability, migration, and certification are not yet one auditable commercial contract.
users:
  - tenant knowledge administrators
  - application operators
  - provider adapter maintainers
affected_surfaces:
  - backend
  - api
  - sdk
  - pc
  - worker
  - database
```

Specs: REQUIREMENTS_SPEC.md, ARCHITECTURE_DECISION_SPEC.md, API_SPEC.md, SDK_SPEC.md,
DATABASE_SPEC.md, MIGRATION_SPEC.md, SECURITY_SPEC.md, PRIVACY_SPEC.md, OBSERVABILITY_SPEC.md,
PERFORMANCE_SPEC.md, TEST_SPEC.md, RELEASE_SPEC.md

## Goals

1. Make catalog tier, executable capability, runtime registration, health, and certification evidence
   agree for every provider.
2. Require an explicit, tenant- and space-scoped provider binding; never select a provider by
   insertion order, source creation time, or an unrelated connector record.
3. Carry authenticated actor, organization, permission, data-scope, trace, and deadline context
   through every external operation without exposing raw credentials.
4. Apply bounded timeout, retry, rate-limit, circuit-breaker, bulkhead, response-size, telemetry,
   and stable error semantics to all provider HTTP clients.
5. Support tested activation, disablement, synchronization, migration, cutover, and rollback
   lifecycles with auditable state transitions.
6. Gate commercial claims on automated contract tests plus live provider, PostgreSQL, security,
   load, failover, rollout, and rollback evidence.

## Non-Goals

- Treat vector databases as full document-management systems when their adapter only proves
  retrieval and point/object reads.
- Advertise LangChain or LlamaIndex as runtime providers before an owned executable adapter ships.
- Store provider secrets in `kb_source` metadata, API responses, logs, metrics, or client state.
- Preserve ambiguous prelaunch provider-selection behavior for compatibility.
- Declare the application commercially released based only on local mocks or SQLite tests.

## Acceptance Criteria

- Every runtime descriptor exposes only capabilities exercised by contract tests; the catalog
  validator rejects tier, category, manifest, crate, wiring, or capability drift.
- Native modes cannot be overridden by connector records. External mode with zero or more than one
  executable provider fails explicitly until one active binding exists.
- Duplicate runtime registration is rejected and cannot overwrite an existing implementation.
- Aggregate provider health checks every configured health-capable provider and reports degraded
  when any required provider is degraded or unavailable.
- Every provider operation receives tenant, organization, actor, permission/data scope, trace id,
  and deadline context, and rejects missing or mismatched scope before network access.
- Provider failures map to a stable taxonomy covering authentication, authorization, rate limit,
  timeout, unavailable/circuit-open, invalid response, unsupported, validation, not found, and
  internal failures, including retryability and `Retry-After` where applicable.
- All provider clients use the approved shared HTTP runtime; direct `reqwest::Client::new()` in
  adapter crates fails a static gate.
- Backend management supports bounded list/retrieve/create/update/test/activate/disable/sync and
  migration status operations through SDKWork v3 envelopes and generated composed SDKs.
- Binding, credential reference, lifecycle, migration, and audit persistence is tenant isolated,
  indexed, RLS-covered for PostgreSQL, and verified on SQLite and PostgreSQL.
- Each adapter tier provider passes deterministic capability, auth, error, timeout, retry,
  rate-limit, body-limit, isolation, and health tests. Production tier additionally requires a
  version-pinned live certification record and licensing approval.
- Retrieval quality gates record Recall@K, MRR/nDCG, citation correctness, empty-query behavior,
  latency, and failure-rate thresholds on a versioned dataset.
- Migration supports dry-run, checkpointed copy/sync where applicable, validation, atomic cutover,
  observation, and rollback without deleting the previous binding or remote data.
- Publication remains inactive until all repository gates and external commercial evidence are
  attached to a release record.

## Non-Functional Requirements

- Security: fail closed on scope or credential resolution; redact secrets and upstream bodies;
  audit binding, test, activation, sync, migration, and rollback commands.
- Privacy: minimize remote payloads, enforce tenant/data scope before egress, and document provider
  data residency, retention, deletion, and subprocessors.
- Reliability: bounded idempotent retry only, circuit breaking, concurrency isolation, cancellation,
  and no false-green readiness.
- Performance: provider-specific SLOs with default connect/request deadlines, bounded result counts
  and response bytes, and no unbounded collection in process memory.

## Trace And Verification

- Decision: `ADR-20260720-knowledge-engine-provider-binding-spi-v2`
- Plan: `PLAN-2026-0720-knowledge-engine-provider-commercialization`
- Review: `REVIEW-20260720-knowledge-engine-provider-commercial-readiness`
- Current automated gates: catalog checker, SPI checker, Rust provider contract tests, route health
  tests, API/envelope/pagination validators, and repository verification.
- Commercial exit additionally requires human architecture/security/data review and real-provider,
  PostgreSQL, load, outage, migration, rollback, licensing, and release evidence.
