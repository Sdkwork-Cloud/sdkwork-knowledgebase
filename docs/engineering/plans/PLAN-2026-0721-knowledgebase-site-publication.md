# PLAN-2026-0721 Knowledgebase Site Publication

Status: in-progress  
Requirement: REQ-2026-0721  
Decision: ADR-20260721-drive-backed-knowledgebase-site-publication  
Owner: SDKWork Knowledgebase maintainers  
Date: 2026-07-21

## Delivery Order

1. Replace legacy contracts with site, release, and host-binding resource models; define bounded
   cursor list operations, idempotent publish, version-fenced update/activation, and rollback.
2. Add SQLite/PostgreSQL schema, indexes, RLS, migration-manifest coverage, and a dedicated SQLx
   store. Remove site persistence from the commerce store and remove `preview_object_key`.
3. Integrate the Drive server uploader and workspace resolver, then build deterministic sanitized
   multi-page releases from published OKF concepts and explicit public assets.
4. Add the public site resolver/router with standalone `/wiki/{knowledgebaseId}` and cloud host
   modes, immutable asset caching, conditional requests, and security headers.
5. Integrate `sdkwork-web-server` through its generated SDK for site/domain/certificate lifecycle.
6. Change the authored App OpenAPI source, materialize it, regenerate SDKs, update route manifests,
   and remove Knowledgebase-owned `upload_sessions`.
7. Update PC upload/editor persistence and add site settings, publication progress, release history,
   host binding, open-site, and rollback workflows using composed SDK methods.
8. Align `etc/`, development orchestration, deployment descriptors, operator runbooks, product and
   architecture documentation, changelog, and release evidence templates.

## Quality Gates

- Contract model and service unit tests cover validation, content selection, deterministic hashes,
  state transitions, idempotency, rollback, and error taxonomy.
- Repository tests cover cursor bounds, tenant isolation, optimistic versions, current-release
  atomicity, cleanup protection, SQLite, PostgreSQL SQL/RLS declarations, and migration manifests.
- Public-route tests cover LAN path shape, host normalization, canonical redirects, traversal,
  encoded paths, private/draft isolation, MIME policy, CSP, ETag, cache policy, and missing assets.
- SDK checks prove authored OpenAPI, materialized authority, generated transports, composed facade,
  and frontend imports agree.
- Frontend tests prove Drive-only upload, stable media identity, failure behavior, publication status,
  host validation, and rollback confirmation.
- Runtime tests prove standalone network bind and `http://<network-ip>:18081/wiki/<id>/`; cloud
  contract tests prove Web Server calls without per-release infrastructure deployment.

## Verification Sequence

Run the narrowest affected crate/package checks first. Then run API envelope, operation, pagination,
SDK generation/consumer, Drive boundary, database migration, source configuration, security,
frontend build, standalone E2E, and cloud adapter integration checks. Only after those pass run the
root `pnpm check`, `pnpm test`, and `pnpm verify` release gates.

Exact commands, output summaries, environment, commit, external dependencies, and any human review
decision are attached to the implementation review and release evidence. A green local mock is not
commercial production evidence.

## Completion Definition

- No production code, API, schema, SDK, UI, config, or active documentation references the legacy
  upload-session or one-file site-deployment design.
- All requirements in REQ-2026-0721 are traced to implementation and automated evidence.
- Production publication stays disabled until PostgreSQL, DNS/TLS, load/SLO, backup/restore,
  rollout, rollback, security/privacy review, and live smoke evidence are independently approved.

