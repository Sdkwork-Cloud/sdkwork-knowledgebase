# REQ-2026-0721 Knowledgebase Site Publication

```yaml
id: REQ-2026-0721
title: Drive-backed knowledgebase website publication
owner: SDKWork Knowledgebase maintainers
status: in-progress
source: product
problem: Authors cannot yet publish an OKF knowledgebase as a secure, versioned, multi-page website with the same behavior in standalone and cloud deployments.
users:
  - knowledge authors
  - tenant knowledge administrators
  - public knowledge readers
  - application operators
affected_surfaces:
  - backend
  - api
  - sdk
  - pc
  - database
  - drive
  - deployment
```

Specs: REQUIREMENTS_SPEC.md, API_SPEC.md, SDK_SPEC.md, DRIVE_SPEC.md, DATABASE_SPEC.md,
MIGRATION_SPEC.md, CONFIG_SPEC.md, DEPLOYMENT_SPEC.md, SECURITY_SPEC.md, PRIVACY_SPEC.md,
PERFORMANCE_SPEC.md, TEST_SPEC.md, RELEASE_SPEC.md

## Goals

1. Make file and rich-media upload use the canonical `sdkwork-drive` uploader lifecycle and store
   only stable Drive URI, space, and node identities in Knowledgebase business state.
2. Publish only explicitly public OKF concepts and their explicitly referenced public Drive assets
   as immutable, searchable, multi-page website releases.
3. Serve standalone sites at `http://<network-ip>:<port>/wiki/<knowledgebaseId>/` and cloud sites at
   `https://<knowledgebaseId>.kb.sdkwork.com/` or a verified custom prefix/domain.
4. Support manual publication by default, optional publication after `okf.concept.published`, atomic
   release activation, release history, and rollback without rebuilding prior releases.
5. Delegate cloud host, DNS, certificate, and reverse-proxy lifecycle to `sdkwork-web-server` while
   Knowledgebase remains the sole owner of site content and release state.

## Non-Goals

- Expose `sources/raw`, generated `output`, `.sdkwork/governance`, drafts, or private Drive nodes.
- Store provider, bucket, object key, presigned URL, or transient download URL in Knowledgebase
  content, site, release, or host-binding tables.
- Use `/wiki` as an internal module, API resource, database table, or Drive namespace name.
- Run arbitrary user JavaScript, active SVG, remote HTML, or untrusted inline event handlers.
- Trigger a web-server infrastructure deployment for every content-only release.

## Functional Requirements

- The browser obtains the upload parent from `spaces.browser.list.data.parentId`, uploads through
  `@sdkwork/drive-app-sdk client.uploader.*`, then submits one stable Drive reference to the
  Knowledgebase import API. Upload failures fail closed; no text-only ingest fallback exists.
- TipTap media nodes persist stable Drive identity and resolve an authorized delivery URL only when
  rendered. Saved documents never contain presigned URLs.
- Each knowledge space has at most one site aggregate. Site configuration includes title,
  visibility, homepage concept, theme, publication mode, canonical host, state, and optimistic
  version.
- A release snapshots the ordered published concept set and content hash, writes immutable HTML,
  assets, search index, sitemap, and manifest artifacts through `sdkwork-drive-uploader-service`,
  and records only stable Drive identities and checksums.
- Activation changes `currentReleaseId` atomically only after all artifacts are ready. Failed
  builds never affect the current release. Rollback atomically selects an earlier ready release.
- A default numeric binding is `<knowledgebaseId>.kb.sdkwork.com`. A verified custom prefix becomes
  canonical and the numeric host redirects to it. Optional external domains require verification
  and certificate activation through `sdkwork-web-server`.
- Standalone delivery resolves `/wiki/{knowledgebaseId}/` and descendant concept paths. Cloud
  delivery resolves the normalized `Host` header and equivalent root-relative paths.
- Public requests return the same not-found behavior for nonexistent, private, unverified, paused,
  or cross-tenant sites.
- Site/release/host-binding management uses SDKWork v3 envelopes, generated SDKs, cursor pagination,
  optimistic versions, idempotency, resource authorization, and mutation audit.

## Non-Functional Requirements

- Security: strict host/path normalization, traversal rejection, MIME allowlist, HTML sanitization,
  CSP, `nosniff`, safe referrer policy, bounded rendering, and no secret or storage topology leak.
- Performance: immutable release assets use long-lived caching; active route resolution is indexed;
  release generation and search-index creation are bounded and streaming where practical.
- Reliability: release creation is idempotent by source content hash, activation is transactional,
  prior releases remain readable for rollback, and cleanup never deletes the current release.
- Privacy: public manifests contain only published page metadata and explicitly public assets.
- Operability: readiness checks verify site storage dependencies, structured telemetry identifies
  site/release/result without unbounded host labels, and standalone binds to an explicitly
  configured network interface.

## Acceptance Criteria

- A user uploads supported files and editor media through Drive and the application persists no
  bucket, object key, presigned URL, or data URL as business authority.
- A published OKF concept tree renders as navigable multi-page HTML with working assets, search,
  sitemap, canonical metadata, and deterministic output checksums.
- Draft, raw-source, governance, and unreferenced Drive content cannot be retrieved from public
  routes, including by guessed node or release identifiers.
- `http://<LAN-IP>:18081/wiki/<knowledgebaseId>/` works in the canonical standalone dev workflow.
- Numeric cloud host, custom prefix canonicalization, redirect alias, external-domain verification,
  and TLS ownership are covered by contract or integration tests.
- A failed publication leaves the active release unchanged; rollback restores the selected release
  without rebuilding it.
- SQLite and PostgreSQL migrations, tenant isolation/RLS, API/SDK generation, frontend SDK-boundary,
  source-config, build, E2E, and release-readiness checks pass with recorded evidence.

## Trace And Verification

- Decision: `ADR-20260721-drive-backed-knowledgebase-site-publication`
- Plan: `PLAN-2026-0721-knowledgebase-site-publication`
- Migration: `MIG-2026-0721-knowledgebase-site-publication`
- Implementation evidence and remaining external release evidence are recorded in the plan and
  release runbook. Commercial publication remains gated until production PostgreSQL, DNS/TLS,
  backup/restore, load, rollout, rollback, and live smoke evidence are attached.

