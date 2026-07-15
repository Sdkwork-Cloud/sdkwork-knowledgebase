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
| Backend design (storage, crates, persistence, API envelope) | [architecture/tech/TECH-2026-06-01-knowledgebase-backend-design.md](architecture/tech/TECH-2026-06-01-knowledgebase-backend-design.md) |
| OKF bundle operator summary | [architecture/tech/TECH-okf-knowledge-bundle.md](architecture/tech/TECH-okf-knowledge-bundle.md) |
| Open API design | [architecture/tech/TECH-2026-06-12-knowledgebase-open-api-design.md](architecture/tech/TECH-2026-06-12-knowledgebase-open-api-design.md) |
| OKF knowledge bundle | [architecture/tech/TECH-2026-06-19-okf-knowledge-bundle-design.md](architecture/tech/TECH-2026-06-19-okf-knowledge-bundle-design.md) |
| Agent RAG design | [architecture/tech/TECH-2026-06-09-knowledgebase-agent-rag-design.md](architecture/tech/TECH-2026-06-09-knowledgebase-agent-rag-design.md) |
| Managed group knowledgebase requirement | [product/requirements/REQ-2026-0713-group-knowledgebase.md](product/requirements/REQ-2026-0713-group-knowledgebase.md) |
| Managed group knowledgebase decision | [architecture/decisions/ADR-20260713-group-knowledgebase-binding-and-launch.md](architecture/decisions/ADR-20260713-group-knowledgebase-binding-and-launch.md) |
| Tenant isolation | [runbooks/tenant-isolation.md](runbooks/tenant-isolation.md) |
| Audit retention | [runbooks/audit-retention.md](runbooks/audit-retention.md) |
| Phase 2 RLS ADR | [architecture/decisions/ADR-20260624-phase2-postgres-rls-multi-tenant.md](architecture/decisions/ADR-20260624-phase2-postgres-rls-multi-tenant.md) |

## Operator runbooks (repository root)

Production launch and backup procedures live under `deployments/runbooks/`:

- [deployments/runbooks/production-launch.md](../deployments/runbooks/production-launch.md)
- [deployments/runbooks/backup-restore.md](../deployments/runbooks/backup-restore.md)

## Allowed content

- Architecture decision records under `docs/architecture/decisions/`
- Active product and architecture docs under `docs/product/` and `docs/architecture/`
- Operator runbooks under `docs/runbooks/`

## Forbidden content

- Root SDKWork standard copies (use `../sdkwork-specs/`)
- Generated SDK transport output
- Secrets, tokens, or customer data
- Active API contracts (use `apis/` and `sdks/`)
- Duplicate historical design copies (canonical architecture lives under `docs/architecture/tech/`; `docs/superpowers/` is redirect-only)
- Active ADR content under retired `docs/adr/`; that directory may contain compatibility redirects only

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
