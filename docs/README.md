# Docs

## Purpose

`docs/` contains repository documentation, architecture records, product requirements, and operator runbooks for SDKWork Knowledgebase.

## Owner

SDKWork Knowledgebase maintainers.

## Canon documents

| Document | Path |
| --- | --- |
| Product PRD | [product/prd/PRD.md](product/prd/PRD.md) |
| MVP launch acceptance | [product/prd/PRD-mvp-launch.md](product/prd/PRD-mvp-launch.md) |
| Phase 2 commercial SaaS | [product/prd/PRD-phase2-commercial-saas.md](product/prd/PRD-phase2-commercial-saas.md) |
| Technical architecture | [architecture/tech/TECH_ARCHITECTURE.md](architecture/tech/TECH_ARCHITECTURE.md) |
| Tenant isolation | [runbooks/tenant-isolation.md](runbooks/tenant-isolation.md) |
| Audit retention | [runbooks/audit-retention.md](runbooks/audit-retention.md) |
| Phase 2 RLS ADR | [adr/ADR-2026-06-24-phase2-postgres-rls-multi-tenant.md](adr/ADR-2026-06-24-phase2-postgres-rls-multi-tenant.md) |

## Operator runbooks (repository root)

Production launch and backup procedures live under `deployments/runbooks/`:

- [deployments/runbooks/production-launch.md](../deployments/runbooks/production-launch.md)
- [deployments/runbooks/backup-restore.md](../deployments/runbooks/backup-restore.md)

## Allowed content

- Architecture decision records under `docs/adr/`
- Active product and architecture docs under `docs/product/` and `docs/architecture/`
- Operator runbooks under `docs/runbooks/`
- Historical implementation notes under `docs/superpowers/` (labeled as history)

## Forbidden content

- Root SDKWork standard copies (use `../sdkwork-specs/`)
- Generated SDK transport output
- Secrets, tokens, or customer data
- Active API contracts (use `apis/` and `sdks/`)

## Related specs

- `../sdkwork-specs/DOCUMENTATION_SPEC.md`
- `../sdkwork-specs/DEPLOYMENT_SPEC.md`
- `../sdkwork-specs/RELEASE_SPEC.md`

## Verification

```bash
pnpm verify
pnpm test:launch-readiness
pnpm lint
```

Status: active.
