# REVIEW-2026-0720 Knowledge Engine Provider Commercial Readiness

Status: open  
Requirement: REQ-2026-0720  
Owner: SDKWork Knowledgebase maintainers  
Reviewed: 2026-07-20  
Scope: catalog, SPI, runtime resolution, health, adapters, security, resilience, observability,
quality, management, migration, certification, and release evidence

## Executive Decision

The repository has a credible modular foundation and now has deterministic P0 catalog, capability,
selection, health, and registration behavior. It is not yet a commercial multi-provider solution.
Current status is **prelaunch-gated / adapter-preview** because explicit binding, actor/data scope,
credential references, shared resilience, management lifecycle, provider observability, migration,
quality evaluation, live certification, and release evidence remain incomplete.

## Maturity Scorecard

| Dimension | Current | Commercial target | Primary gap |
| --- | ---: | ---: | --- |
| Catalog and capability truth | 4/5 | 5/5 | Keep machine checks authoritative |
| Deterministic selection | 3/5 | 5/5 | Persisted explicit binding |
| SPI semantic completeness | 2/5 | 5/5 | Context, optional capabilities, typed lifecycle |
| Tenant/security isolation | 2/5 | 5/5 | Actor/data scope and credential reference boundary |
| Resilience | 1/5 | 5/5 | Shared timeout/retry/circuit/bulkhead/body limits |
| Observability | 2/5 | 5/5 | Per-operation metrics/traces and safe failure detail |
| Management lifecycle | 1/5 | 5/5 | Test/activate/disable/sync/status UI and API |
| Migration and rollback | 0/5 | 5/5 | Durable state machine and cutover evidence |
| Retrieval quality | 1/5 | 5/5 | Versioned dataset and quantitative thresholds |
| Provider certification | 1/5 | 5/5 | Live version matrix, licensing and SLO evidence |

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

## Open Findings

| Priority | Finding | Risk | Required closure |
| --- | --- | --- | --- |
| P0 | No persisted explicit provider binding | nondeterministic lifecycle and no safe cutover | accepted ADR, binding aggregate, migration |
| P0 | SPI lacks actor/permission/data scope | cross-tenant or over-broad remote access | fail-closed execution context |
| P0 | Credentials are environment/file based per adapter | weak tenant rotation and audit boundary | approved credential-reference service |
| P0 | Adapter HTTP clients lack unified deadlines and isolation | hangs, retry storms, cascading outage | shared resilience runtime |
| P1 | Error taxonomy collapses failures into internal errors | unsafe retry and poor operations | stable categorized provider error |
| P1 | External lifecycle trait is erased by registry | sync/test cannot be managed generically | typed SPI v2 handles |
| P1 | Source API is list/create only | no test/disable/update/sync workflow | management API/SDK/UI |
| P1 | No durable migration/cutover/rollback state | provider switching can lose availability | checkpointed migration operation |
| P1 | Provider metrics and traces are insufficient | slow incident detection and diagnosis | per-provider operation telemetry |
| P1 | No quantitative retrieval evaluation | regressions can ship undetected | versioned evaluation suite |
| P1 | No live provider/version certification | mocks do not prove upstream compatibility | certification matrix and release gate |
| P2 | Catalog metadata and executable registry share concepts | discoverability can be mistaken for readiness | separate discovery and runtime projections |
| P2 | Operator runbooks lack provider-specific failure modes | inconsistent recovery | test/sync/outage/rollback runbooks |

## Evidence Reviewed

- Machine catalog and all vendor manifests.
- Core contract, service registry/resolver, runtime wiring, provider health route, and ten adapter
  crates including HTTP mock tests.
- Local SPI specification, PRD, technical architecture, security/privacy/observability/performance,
  API/SDK/database/migration/test, and release standards.
- Focused passing evidence: catalog and SPI checkers; contract tests; resolver/catalog/native/
  registry tests; 59 tests across all ten adapter crates; provider-health normal and
  external-degraded route tests.

## Commercial Blockers And Required Review

Commercial completion cannot be claimed until ADR-20260720 is accepted and implemented, real
provider certification and release-environment PostgreSQL/load/outage/migration/rollback evidence
is attached, licensing/security/privacy review is approved, and existing application publication
gates are deliberately activated through release governance. Local code changes cannot manufacture
those external facts.
