# REVIEW-20260721 Live Wiki Deployment Integration Readiness

Status: blocked-design-only
Owner: SDKWork Knowledgebase maintainers
Date: 2026-07-21
Requirement: REQ-2026-0721
Decision: ADR-20260721-live-mounted-wiki-publication (proposed)
Machine contract: `specs/live-wiki-publication.spec.json`
Specs: REQUIREMENTS_SPEC.md, ARCHITECTURE_DECISION_SPEC.md, API_SPEC.md, SDK_SPEC.md,
EVENT_SPEC.md, DATABASE_SPEC.md, DEPLOYMENT_SPEC.md, RELEASE_SPEC.md, SECURITY_SPEC.md,
PERFORMANCE_SPEC.md, TEST_SPEC.md

## 1. Scope And Method

This review checks the current working implementation, not only the proposed PRD. It inspects the
Knowledgebase, Drive, Deployments, and Web Server application manifests, component contracts,
OpenAPI authorities, database baselines, Rust services/routes, frontend publication UI, event
producers/consumers, certificate behavior, and tests.

The reviewed end-to-end outcome is:

```text
Drive sources/raw commit
  -> Knowledgebase source projection and publication command
  -> Knowledgebase typed provider and versioned change event
  -> Deploy-owned active Site Resource/Mount/Binding/SiteRevision
  -> Web Server WIKI provider adapter, cache invalidation and public response
```

## 2. Verdict

The target architecture is coherent, but the current repositories do not provide an integrated or
realtime Wiki publication capability. The existing system must not be described as live, publicly
deployable, production-ready, or commercially ready.

What exists today is limited to Drive-backed `sources/raw` browsing/upload foundations,
Knowledgebase ingestion and a generic outbox, release-oriented Deploy APIs, duplicated Web Server
control-plane APIs, and partial certificate management. The canonical WikiPublication, provider,
content events, descriptor runtime, and public WIKI handler remain design-only.

## 3. Current Evidence

| Surface | Current evidence | Result |
| --- | --- | --- |
| Contract status | `specs/README.md` states the Live Wiki contract is proposed and is not implementation authority before approval | design only |
| Application release | `sdkwork.app.config.json` is prelaunch-gated with missing release and production evidence | blocked |
| Wiki API | Knowledgebase app/open/backend OpenAPI has no WikiPublication, Wiki Site Resource, provider validate/open, or Wiki event operations | absent |
| Wiki schema | Knowledgebase baseline and schema contract have no canonical WikiPublication/source-publication projection tables | absent |
| Source update | `crates/sdkwork-knowledgebase-drive/src/adapter.rs` rejects an existing `sources/raw` logical path as immutable | conflicts with edits/republish |
| Outbox | the implemented producer emits `knowledge.ingest.succeeded`; no Wiki provider/public-route event is produced | insufficient |
| Component events | Knowledgebase and Drive root component contracts declare empty event inventories | absent contract |
| Publish UI | `PublishModal.tsx` implements WeChat distribution, contains disabled third-party channels and has no Wiki/Site/Mount/Domain workflow | wrong product surface |
| Deploy source model | `CreateDeploymentRequest` optionally references `releaseId`; no Site Resource/Variant/Mount/Binding/SiteRevision API or tables exist | release-oriented |
| Dual authority | Deploy owns writable `deploy_site/domain/certificate/deployment`; Web Server owns writable parallel `web_*` tables and overlapping app APIs | unsafe split brain |
| Web data plane | no WebsiteRuntimeDescriptor ingestion, Drive/Knowledgebase provider adapter, WIKI handler, provider event checkpoint, or public content tests exist | absent |
| Provider SDK wiring | Deploy does not declare a Knowledgebase SDK; Web Server does not declare Drive or Knowledgebase provider SDKs | disconnected |
| Managed TLS | certificate renew only sets `renewal_status=planned`; OpenAPI states the ACME worker is not online | not automatic |
| E2E evidence | no upload-to-public, update-to-public, revoke-to-not-public, event-gap, cache, or multi-Site provider-reuse test exists | absent |

## 4. P0 Blocking Findings

### P0-1 WikiPublication Aggregate Is Not Implemented

There is no canonical publication row, per-file source/publication/visibility projection, provider
generation, page public version, publication command API, provider API, or migration. Existing OKF
concept publication is a separate semantic capability and cannot authorize public Wiki delivery.

Closure requires accepted schema/API/SDK ownership, dual-engine migrations, idempotent provisioning
for every Knowledgebase, backfill, lifecycle fencing, and negative tenant/visibility tests.

### P0-2 Current Source Immutability Blocks Realtime Updates

The current adapter rejects a write when the same `sources/raw` logical object already exists. A
Wiki must instead keep a stable Drive node while each edit creates a new immutable Drive version.
Physical version/blob immutability remains mandatory; logical path immutability is forbidden.

Closure requires update/version, rename/move, delete/restore, concurrent write, rollback and old/new
route invalidation tests using stable node and version identities.

### P0-3 Drive-To-Knowledgebase Change Stream Is Missing

There is no accepted Drive AsyncAPI contract, root-scoped subscription, durable Knowledgebase
checkpoint, replay protocol, or reconciliation implementation for `sources/raw`. The current
Knowledgebase outbox publishes ingestion success after local work; it does not consume Drive node
version/path/security lifecycle as the publication source.

Closure requires the versioned Drive input event types named by the machine contract, at-least-once
delivery, idempotency, ordering fences, gap detection, dead letter, replay, and bounded root scan.

### P0-4 Knowledgebase Provider Contract And SDK Are Missing

Deploy and Web Server cannot validate or open a canonical Wiki resource. The required authority is
a Knowledgebase-owned `internal-api` and generated `sdkwork-knowledgebase-internal-sdk`, with an
equivalent typed Rust port in standalone topology. Raw HTTP, manual service headers, direct
Knowledgebase database access, and an anonymous Knowledgebase public router are forbidden.

### P0-5 Deployments Remains Release-Oriented

Deploy currently creates deployments around optional `releaseId` and owns `deploy_release`; it has
no discriminated `DRIVE_DIRECTORY`/`KNOWLEDGEBASE_WIKI` source resolver, Site Resource, Variant,
Mount, Binding, immutable SiteRevision, or runtime descriptor persistence/distribution.

Closure requires the proposed live-resource schema and owner SDK integrations while retaining
Release only for frozen package/Git/image workflows.

### P0-6 Deployments And Web Server Are Competing Control Planes

Both repositories expose writable Site/Domain/Deployment management and persist near-equivalent
tables. No public cutover is safe while both can write. Deployments must become the only Site,
Domain, Certificate-policy and Revision authority. Web Server must retain only immutable runtime/TLS
snapshots, provider checkpoints, bounded cache metadata, observations and usage spool state.

Closure requires shadow comparison, migration reconciliation, a single-writer fence, removal of Web
app-api write routes, retirement of `web_site/domain/deployment/certificate` authority, and rollback
evidence that cannot reactivate dual writers.

### P0-7 Web Server Has No WIKI Delivery Runtime

The compiled descriptor validator/index, atomic active pointer, WIKI provider adapter, public route
resolution, pinned representation streaming, safe render response, provider event consumer and
route-scoped cache invalidation are absent. Existing Nginx configuration synchronization is useful
infrastructure but is not the proposed website runtime.

### P0-8 Realtime Publication Events Are Missing

Knowledgebase does not atomically commit public state plus a Wiki outbox event. Web Server does not
consume provider generation/page version events. Deploy must not be inserted into this content hot
path: it compiles configuration revisions, while Knowledgebase emits provider events directly to Web
Server consumers.

### P0-9 Automatic Certificate Renewal Is Not Closed

The current renew operation schedules state only. There is no ACME account/order/challenge worker,
DNS-01 provider, HTTP-01 routing, KMS-backed key issuance, certificate version activation, fleet
distribution, served-SNI convergence, retry/escalation, or expiry drill evidence.

### P0-10 User And Admin Workflows Do Not Match The Target

The existing Publish modal is a third-party content distribution surface, not Wiki publication.
The required Wiki overview, source state explorer, page preview/publish controls, Site attachment,
Mount/domain/TLS status, event lag, cache purge/reconcile, failed job and provider health views are
not implemented through generated SDKs.

## 5. Realtime Capability Standard

Realtime means event-driven bounded eventual visibility. It does not mean unsafe publication at
upload completion, synchronous conversion of every format, or bypassing review/security gates.

| Workflow | Required public behavior |
| --- | --- |
| `REVIEW_REQUIRED` first upload | author state and private preview update promptly; public content changes only after a version-fenced publish command |
| `REVIEW_REQUIRED` published-page edit | prior verified public version remains or is removed per explicit policy; new version requires republish |
| `AUTO_PUBLIC_AFTER_CHECKS` native update | new version publishes automatically after scan, parse/render, route and policy gates |
| conversion-required update | asynchronous progress; no synchronous freshness claim; last verified public policy remains explicit |
| private/quarantine/delete/unpublish | priority revocation; stale public delivery is forbidden regardless of processor or event backlog |

Target measurements are defined in the machine contract. A realtime claim requires native
auto-public Drive-commit-to-public p95/p99 evidence, explicit-publish-to-public evidence, priority
revocation evidence, and event-lag/gap dashboards in both standalone and cloud topologies.

## 6. Required Generation And Event Model

The implementation must keep these identities separate:

- Drive source checkpoint: ingestion/reconciliation progress, never a public cache key;
- provider generation: provider-wide eligibility/root/policy fence;
- page public version: per-route public representation and revocation fence;
- navigation generation: navigation snapshot version;
- search generation: public search snapshot version;
- SiteRevision policy generation: Deploy-owned configuration version.

Private processing must not flush public caches. An ordinary page update invalidates only affected
routes plus required navigation/search snapshots. Provider-wide generation changes are reserved for
provider-wide eligibility or policy transitions.

## 7. Implementation And Launch Gates

1. Accept the four repository ADRs, exact source/public/internal API names, AsyncAPI event schemas,
   database contracts and no-compatibility prelaunch migration.
2. Implement Drive change events/subscriptions and logical-update-to-immutable-version semantics.
3. Implement canonical WikiPublication, projection workers, commands, internal SDK and provider
   events with reconciliation.
4. Implement Deploy Site Resource/Variant/Mount/Binding/SiteRevision and descriptor compiler;
   migrate to one control-plane writer.
5. Implement Web descriptor activation, WIKI provider adapter, provider-event checkpoints, cache,
   stream delivery and TLS snapshot execution.
6. Implement managed certificate orchestration and prove renewal/rotation/expiry recovery.
7. Build generated-SDK-backed user/admin UI and close permission, accessibility and error states.
8. Pass contract, dual-engine, tenant isolation, security, E2E, load/soak, event fault, backup,
   rollout and rollback evidence.

## 8. Claim Policy

Until every P0 finding is closed with executable evidence:

- the Live Wiki contract remains `proposed`;
- no repository may claim that Wiki public deployment is implemented;
- no repository may claim upload/update-to-public realtime behavior;
- no commercial or production launch may rely on the target architecture;
- preview or ingestion success must not be presented as public publication success.
