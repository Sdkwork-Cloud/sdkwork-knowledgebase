# APIs

## Purpose

`apis/` contains authored API contract sources, route materialization inputs, examples, changelogs, and validation fixtures for SDKWork Knowledgebase.

## Owner

SDKWork Knowledgebase maintainers.

## Allowed Content

- OpenAPI contract sources under `open-api/`, `app-api/`, `backend-api/`.
- API examples, changelogs, and validation fixtures.
- RPC/async contract sources when added.

## Forbidden Content

- Generated SDK transport output (belongs in `sdks/`).
- SDK family directories.
- Controller/handler/service/repository implementation code.
- Runtime state, secrets, and generated SDK control-plane `.sdkwork/` files.

## Related Specs

- `../sdkwork-specs/API_SPEC.md`
- `../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md`
- `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`

## Verification

Run from the repository root:

```powershell
powershell -ExecutionPolicy Bypass -File tools/verify_sdkwork_structure.ps1
powershell -ExecutionPolicy Bypass -File tools/verify_openapi_operation_ids.ps1
```

## Current State

SDK family OpenAPI authorities under `sdks/*/openapi/` are the **generation source of truth**. The `apis/` directory holds synchronized review copies materialized by `tools/materialize-apis-authority.mjs` (`sdks/` → `apis/`). Edit OpenAPI in `sdks/`, run `pnpm api:materialize`, and commit both trees when contracts change.

Governance metadata is applied by:

- `tools/apply-knowledgebase-openapi-auth-mode.mjs` — `x-sdkwork-auth-mode`
- `tools/apply-knowledgebase-openapi-permissions.mjs` — backend `x-sdkwork-permission`
- `sdks/standardize-knowledgebase-sdk-family.mjs` — SDK family assembly and open-api derivation

### API Surface Inventory

| Surface | Authority | OpenAPI Location | SDK Family |
| --- | --- | --- | --- |
| Open API | `sdkwork-routes-knowledgebase-open-api` | `sdks/sdkwork-knowledgebase-sdk/openapi/knowledgebase-open-api.openapi.json` | `sdkwork-knowledgebase-sdk` |
| App API | `sdkwork-routes-knowledgebase-app-api` | `sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json` | `sdkwork-knowledgebase-app-sdk` |
| Backend API | `sdkwork-routes-knowledgebase-backend-api` | `sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json` | `sdkwork-knowledgebase-backend-sdk` |
