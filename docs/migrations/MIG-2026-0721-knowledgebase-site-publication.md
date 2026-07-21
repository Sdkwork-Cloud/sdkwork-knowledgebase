# MIG-2026-0721 Knowledgebase Site Publication

Status: in-progress  
Requirement: REQ-2026-0721  
Decision: ADR-20260721-drive-backed-knowledgebase-site-publication  
Owner: SDKWork Knowledgebase maintainers  
Date: 2026-07-21

## Scope

Replace the prelaunch `kb_site_deployment` model with `kb_site`, `kb_site_release`, and
`kb_site_host_binding`. Remove Knowledgebase-owned upload-session API ownership and all business
persistence of provider/bucket/object-key/presigned-URL identities.

## Schema Changes

- `kb_site`: tenant, organization, knowledge space, title, visibility, homepage concept, theme,
  publish mode, status, canonical host binding, current release, optimistic version, timestamps.
- `kb_site_release`: immutable site/content identity, state, source revision/hash, Drive manifest
  space/node/URI, checksum, page/asset counts, predecessor, error metadata, version, timestamps.
- `kb_site_host_binding`: site, binding type, normalized prefix/hostname, canonical/verification/
  activation state, Web Server site/domain/deployment references, version, timestamps.
- Unique and lookup indexes enforce one site per tenant/space, unique normalized host identity,
  release content idempotency, and bounded current-release/host resolution.
- PostgreSQL row-level security covers all three tables with the established tenant setting.

## Prelaunch Data Policy

The application has not launched and the existing deployment records contain only preview object
keys that violate the new Drive ownership contract. They are not migrated or dual-read. The legacy
table is dropped after the new tables are created. Authors republish from current published OKF
concepts; this produces valid Drive-backed releases and auditable host bindings.

## Rollout

1. Back up the database and record migration/version checksums.
2. Apply SQLite and PostgreSQL migrations in a non-production environment.
3. Verify indexes, foreign keys, RLS/policies, schema manifest, and empty legacy table absence.
4. Deploy the API/worker/public-router release that understands only the new schema.
5. Create default numeric host bindings and publish representative sites.
6. Verify standalone access, cloud host/domain/TLS integration, cross-tenant denial, rollback, and
   release cleanup protection before enabling traffic.

## Failure And Recovery

- Before application rollout, restore the captured database backup if migration validation fails.
- After new site writes, do not downgrade to the legacy schema. Fix forward, or restore the complete
  pre-rollout database and Drive snapshot as one consistency unit.
- A failed release build remains non-current and can be retried idempotently by content hash.
- A bad content release is recovered by selecting the previous ready release. A bad host change is
  rolled back through Web Server and the previous canonical binding is restored.

## Verification Evidence

Required evidence includes SQLite and PostgreSQL migration tests, RLS/tenant-isolation tests,
idempotent replay, backup/restore rehearsal, old-table/column absence checks, current-release
transaction tests, Drive manifest resolution, public-route smoke tests, and operator sign-off.

