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

API authority OpenAPI files are materialized under `sdks/` SDK family directories. This `apis/` directory is the canonical source location for authored API contracts. As the project matures, OpenAPI authority files should be authored here and materialized to `sdks/` for SDK generation.

### API Surface Inventory

| Surface | Authority | OpenAPI Location | SDK Family |
| --- | --- | --- | --- |
| Open API | `sdkwork-knowledgebase-open-api` | `sdks/sdkwork-knowledgebase-sdk/openapi/knowledgebase-open-api.openapi.json` | `sdkwork-knowledgebase-sdk` |
| App API | `sdkwork-knowledgebase-app-api` | `sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json` | `sdkwork-knowledgebase-app-sdk` |
| Backend API | `sdkwork-knowledgebase-backend-api` | `sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json` | `sdkwork-knowledgebase-backend-sdk` |
