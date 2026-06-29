> Owner: SDKWork Knowledgebase maintainers  
> Status: **completed (2026-06-24)** — historical design record only.

## Summary

This document recorded the migration from legacy `services/` layout and pre-standard crate names to the current SDKWork application-root structure. That migration is **complete**.

## Current canonical references

- Repository layout and crate map: [TECH_ARCHITECTURE.md](TECH_ARCHITECTURE.md)
- Implementation record and verification commands: [TECH-2026-06-11-sdkwork-structure-standardization-implementation.md](TECH-2026-06-11-sdkwork-structure-standardization-implementation.md)
- Backend/API envelope and SDK surfaces: [TECH-2026-06-01-knowledgebase-backend-design.md](TECH-2026-06-01-knowledgebase-backend-design.md)

## Final shape (as implemented)

| Responsibility | Crate |
| --- | --- |
| Object key planning | `sdkwork-intelligence-knowledgebase-object-key-service` |
| Business services / ports | `sdkwork-intelligence-knowledgebase-service` |
| SQLx repositories | `sdkwork-intelligence-knowledgebase-repository-sqlx` |
| App API routes | `sdkwork-routes-knowledgebase-app-api` |
| Backend API routes | `sdkwork-routes-knowledgebase-backend-api` |
| Open API routes | `sdkwork-routes-knowledgebase-open-api` |
| PC browser/desktop surface | `apps/sdkwork-knowledgebase-pc/` |

## Verification

```bash
pnpm check
pnpm api:materialize:check
pnpm sdk:generate:check
node ../sdkwork-specs/tools/check-api-response-envelope.mjs --workspace .
cargo test -p sdkwork-routes-knowledgebase-app-api -p sdkwork-routes-knowledgebase-backend-api -p sdkwork-routes-knowledgebase-open-api
```

Do not reintroduce legacy package names, `services/` layout, compatibility aliases, or duplicate standard text from `../sdkwork-specs/`.
