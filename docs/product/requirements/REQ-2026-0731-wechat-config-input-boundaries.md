# REQ-2026-0731 WeChat Configuration Input Boundaries

id: REQ-2026-0731
title: Bound and validate WeChat configuration replacement input
owner: SDKWork maintainers
status: accepted
source: security
problem: The app-api schemas and Rust configuration service accept weakly constrained WeChat account and applet objects, allowing direct API clients to bypass frontend limits and submit oversized fields, arrays, unknown properties, or embedded media payloads.

## Goals

- Keep every Official Account and Applet replacement request bounded before Drive persistence.
- Make OpenAPI constraints and Rust service validation equivalent for counts, lengths, enums, and embedded verification text.
- Reject browser Data URLs and bare media URLs in avatar fields until a managed Drive `MediaResource` contract exists.
- Preserve the existing app-api paths, operation ids, authentication, tenant isolation, and 1 MiB aggregate Drive object limit.

## Non-Goals

- Adding custom avatar upload or a new Drive media contract.
- Enabling WeChat article publish/preview while their existing launch gates remain open.
- Adding optimistic concurrency to configuration replacement.
- Changing WeChat provider credentials, secret encryption, or deployment configuration.

## Acceptance Criteria

- OpenAPI request/resource schemas declare `additionalProperties: false`, required fields, bounded strings and arrays, and supported enum values.
- Replacement arrays reject more than 100 entries; domain arrays reject more than 50 entries.
- Domain verification filenames are at most 255 characters and content is at most 65,536 UTF-8 bytes in the Rust service.
- Avatar values are non-empty bounded icon values and reject `data:` values and URI-style media references.
- Secret inputs remain write-only in OpenAPI and continue to be encrypted at rest and redacted from list/update responses.
- Unknown object fields and all invalid values fail before Drive writes with the existing invalid-request mapping.
- Service and route tests cover accepted boundaries and representative overflow, enum, unknown-field, media-identity, and no-write-on-rejection cases.
- App SDK OpenAPI and generated SDK artifacts are materialized/generated only from the authored app-api contract.

## Accepted Contract

| Input | Boundary |
| --- | --- |
| Official Accounts / Applets per replacement | 100 each |
| Domain values per domain array | 50 |
| Resource id / name | 128 characters |
| Avatar icon value | 32 characters; no URI scheme or slash-based media path |
| App ID | 64 characters |
| Description | 2,048 characters |
| App secret / message token | 256 characters |
| Encoding AES key | 43 characters maximum |
| Verification filename | 255 characters |
| Verification content | 65,536 characters in OpenAPI and 65,536 UTF-8 bytes in Rust |
| Official Account domain value | 255 characters |
| Applet endpoint value | 2,048 characters |
| Applet path | 1,024 characters; empty selects the default homepage |

Supported values are `subscription | service` for Official Account type,
`plain | compatible | safe` for encryption mode, and `json | xml` for message format. Unknown
object fields are rejected. Secret properties are write-only; rotating one secret preserves each
independently omitted token or AES value, while responses remain redacted.

The authored authority is
`sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json`.
`pnpm api:materialize` derives `apis/app-api/knowledgebase-app-api.openapi.json`, and SDK generation
derives `generated/server-openapi`; neither derived surface is an independent authoring source.

## Non-Functional Requirements

- Security: follow `SECURITY_SPEC.md` input/output safety and fail closed before persistence.
- Privacy: do not log configuration payloads or secret values.
- Performance: validation is linear in the bounded request size and does not clone payloads solely for validation.
- Reliability: rejected requests perform no Drive write; the existing 1 MiB aggregate object limit remains the final persistence bound.

## Affected Surfaces

- api
- sdk
- backend
- pc

## Trace

Specs:

- `REQUIREMENTS_SPEC.md`
- `API_SPEC.md`
- `WEB_BACKEND_SPEC.md`
- `WEB_FRAMEWORK_SPEC.md`
- `SDK_SPEC.md`
- `SDK_WORKSPACE_GENERATION_SPEC.md`
- `SECURITY_SPEC.md`
- `RUST_CODE_SPEC.md`
- `TEST_SPEC.md`

Components:

- `apis/app-api/knowledgebase-app-api.openapi.json`
- `crates/sdkwork-knowledgebase-contract`
- `crates/sdkwork-intelligence-knowledgebase-service`
- `crates/sdkwork-routes-knowledgebase-app-api`
- `sdks/sdkwork-knowledgebase-app-sdk`
- `apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase`

## Verification

- `cargo test -p sdkwork-intelligence-knowledgebase-service wechat::config_store`
- `cargo test -p sdkwork-routes-knowledgebase-app-api --test integration_wechat_routes`
- `pnpm api:materialize:check`
- `pnpm check`
- `node ../sdkwork-specs/tools/check-application-layering.mjs --root .`
- `git diff --check`

## Acceptance Evidence

Verified on 2026-07-31:

- WeChat configuration service tests: 10 passed, including bounded input, no-I/O rejection, secret
  encryption, and independent secret preservation.
- App route integration tests: 3 passed through the real Axum route, real WeChat service, and the
  standard bounded Drive test port; Problem Details use numeric `40001` / `50001` codes and UUID
  `traceId` values.
- App API contract tests: 3 passed, including bounded WeChat configuration schemas.
- Security alignment: 36 passed; Knowledgebase PC feature package: 68 passed.
- `pnpm api:materialize:check`, SDK ownership, pagination, application layering, Rust backend
  composition, generated TypeScript SDK build, composed SDK typecheck, and `pnpm check` passed.
- Re-running `node tools/knowledgebase_sdk_generate.mjs --family sdkwork-knowledgebase-app-sdk`
  reported `Impact: none` with zero generated file changes.
