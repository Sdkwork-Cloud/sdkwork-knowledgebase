> Owner: SDKWork maintainers  
> Status: **completed** — public open-api surface, SDK family, and gateway registration are live.

**Goal:** Add the Knowledgebase public open-api surface at `/knowledge/v3/api`, generate the owner-only `sdkwork-knowledgebase-sdk` family metadata, and register the surface in `sdkwork-api-cloud-gateway`.

**Outcome:** `sdkwork-routes-knowledgebase-open-api`, materialized OpenAPI authority, generated TypeScript SDK, cloud gateway config bundles, and smoke/contract guards.

**Verification:**

```bash
pnpm sdk:check
pnpm api:materialize:check
pnpm test:launch-readiness
node --test scripts/smoke-knowledgebase-open-api.test.mjs
```

**Design reference:** [TECH-2026-06-12-knowledgebase-open-api-design.md](TECH-2026-06-12-knowledgebase-open-api-design.md)
