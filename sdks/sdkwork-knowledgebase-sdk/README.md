# sdkwork-knowledgebase-sdk

Domain: intelligence
Capability: knowledgebase
Package type: sdk-family
Status: standardized

This README is the SDKWork module entrypoint for `sdkwork-knowledgebase-sdk`. The machine-readable component contract is `specs/component.spec.json`; canonical standards are under `../../../sdkwork-specs/`.

## Public API

- Open API authority: `sdkwork-knowledgebase-open-api`
- Open API prefix: `/knowledge/v3/api`
- Generated TypeScript package: `@sdkwork/knowledgebase-sdk`

## Required SDK Surface

- `SdkworkKnowledgebaseClient`

## Configuration

Protected open-api consumers must use an API key credential provider for this SDK family. They must not reuse app login TokenManager credentials or assemble manual API key headers in feature code.

## Security

Do not add secrets, live tokens, manual auth headers, or app-local credential handling to this module. Protected open-api access must use generated SDK credential infrastructure or an approved service boundary declared in the component contract.

## Extension Points

Extension points are limited to public exports, runtime entrypoints, SDK clients, events, and config keys declared in `specs/component.spec.json`.

## Verification

- `node sdks/standardize-knowledgebase-sdk-family.mjs --check`
- `node sdks/test/verify-sdk-ownership-boundaries.test.mjs`

## Owner And Status

Owner and lifecycle status are tracked in `specs/component.spec.json`. Update that contract before changing public integration behavior.
