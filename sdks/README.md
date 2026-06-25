# SDKs

## Purpose

`sdks/` contains SDKWork Knowledgebase SDK family workspaces, OpenAPI authority materialization outputs, derived generator inputs, generated SDK evidence, and route manifest artifacts.

## Owner

SDKWork Knowledgebase maintainers.

## Allowed Content

- SDK family roots such as `sdkwork-knowledgebase-app-sdk/` and `sdkwork-knowledgebase-backend-sdk/`.
- OpenAPI authority files and derived SDK generator inputs owned by SDKWork Knowledgebase.
- Generated SDK control-plane evidence under generated SDK output roots.
- Normalized route manifests under `_route-manifests/`.
- Deterministic SDK verification scripts.

## Forbidden Content

- Authored API contract sources that belong under `apis/`.
- Runtime service, route, repository, or application implementation code.
- Generated SDK transport output inside `apis/`.
- Local `node_modules/`, `dist/`, `package-lock.json`, or other build artifacts.
- Hand-edited generated SDK transport files.

## Related Specs

- `../sdkwork-specs/SDK_SPEC.md`
- `../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md`
- `../sdkwork-specs/API_SPEC.md`
- `../sdkwork-specs/TEST_SPEC.md`

## Verification

Run from the repository root:

```powershell
node sdks/standardize-knowledgebase-sdk-family.mjs --check
node sdks/test/verify-sdk-ownership-boundaries.test.mjs
powershell -ExecutionPolicy Bypass -File tools/verify_sdkwork_structure.ps1
```
