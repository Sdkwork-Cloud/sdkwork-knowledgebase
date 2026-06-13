# sdkwork-router-knowledgebase-open-api

Domain: intelligence
Capability: knowledgebase
Package type: rust-route-crate
Surface: open-api

This crate owns the SDKWork Knowledgebase public open-api route adapter for `/knowledge/v3/api`.

## Responsibilities

- Mount public Knowledgebase open-api routes.
- Expose deterministic route manifest metadata.
- Decode HTTP requests, consume typed API key context, call injected service traits, and map responses to API contracts.

## Boundaries

- Does not own business rules, SQLx queries, or provider clients.
- Does not expose login, session, app-api, or backend-api routes.
- Does not import generated SDK output for the same authority.

## Verification

- `cargo test -p sdkwork-router-knowledgebase-open-api`
- `powershell -ExecutionPolicy Bypass -File tools/verify_sdkwork_structure.ps1`
