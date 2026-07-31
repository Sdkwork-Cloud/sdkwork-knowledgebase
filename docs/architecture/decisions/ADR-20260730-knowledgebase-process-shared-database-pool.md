# ADR-20260730: Knowledgebase Process-Shared Database Pool

- Status: temporary exception
- Date: 2026-07-30
- Owners: sdkwork-knowledgebase maintainers
- Authority: `sdkwork-specs/DATABASE_SPEC.md` section 32 and `sdkwork-specs/DATABASE_SPEC_PROCESS_SHARED_POOL.md`

## Context

The API gateway, maintenance worker, and internal group-lifecycle RPC host embed Knowledgebase,
Drive, ID allocation, and, for the gateway, IAM database consumers. Independent pool creation
multiplied `SDKWORK_DATABASE_MAX_CONNECTIONS` by module and replica. The Knowledgebase and embedded
Drive repositories still expose `sqlx::AnyPool`, while pgvector and framework lifecycle consumers
use typed `sqlx::PgPool`.

## Decision

Every long-running Knowledgebase process owns one canonical typed PostgreSQL pool. Drive, pgvector,
and cloud provider resolution clone that handle. One temporary `sqlx::AnyPool` compatibility pool
is allowed for the remaining repositories. Runtime bootstrap divides the single
`SDKWORK_DATABASE_MAX_CONNECTIONS` process budget across those two drivers; an odd connection is
assigned to the typed pool, and values below two fail startup. Modules may clone handles but may not
construct independent pools.

PostgreSQL connection options include deployment-owned `app.current_tenant_id` and
`app.current_organization_id`. Caller-supplied or duplicate `options` values that could override
either value fail closed. The App API validates its runtime tenant argument against the same fixed
scope before creating stores; internal RPC rejects scope mismatches before constructing repository
or Drive dependencies. This is valid only for the supported one-tenant/organization-per-process
deployment model. It does not approve request-shared multi-tenant pooling.

## Operational Contract

- `SDKWORK_DATABASE_TEMPORARY_ANY_POOL_EXCEPTION=true` enables the declared compatibility driver.
- `SDKWORK_DATABASE_TEMPORARY_DRIVER_POOL_COUNT=1` fixes the number of temporary driver pools.
- `SDKWORK_DATABASE_MAX_CONNECTIONS` is the combined per-process budget, not a per-module value.
- Drive, pgvector, and cloud provider resolution reuse the canonical typed pool handle.
- A normalized database identity mismatch fails startup instead of creating another pool.
- PostgreSQL pool or pgvector initialization failure fails readiness and startup.

## Removal Milestone

Before the first commercial production release, authoritative PostgreSQL Knowledgebase and embedded
Drive repositories must migrate from `sqlx::AnyPool` to typed `sqlx::PgPool` or the framework
`DatabasePool`. The temporary environment keys, exception entries, and this temporary status are
removed in the same change. SQLite remains a test/client-local fixture and is not part of the
server migration.

## Consequences

Connection growth is bounded per process and therefore scales predictably with replica count.
Deployment tenant RLS context applies when each physical connection is established. Until the
removal milestone is complete, two physical driver pools share one budget; release readiness must
continue to report the temporary exception rather than claiming a single physical pool.
