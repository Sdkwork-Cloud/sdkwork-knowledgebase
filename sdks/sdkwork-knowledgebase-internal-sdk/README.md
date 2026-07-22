# SDKWork Knowledgebase Internal SDK

Generated clients for the ingress-token-protected Knowledgebase Internal API.

- Authority: `openapi/sdkwork-knowledgebase-internal-api.openapi.yaml`
- Derived generator input: `openapi/sdkwork-knowledgebase-internal-api.sdkgen.yaml`
- Prefix: `/internal/v3/api`
- Surface: `internal`
- Owner: `sdkwork-knowledgebase`

## Operations

The authority and both generated transports expose exactly six owner operations:

| Operation id | Method and path | Purpose |
| --- | --- | --- |
| `driveEvents.receive` | `POST /internal/v3/api/knowledgebase/drive_events` | Receive authenticated Drive node/version lifecycle events. |
| `wikiPublications.retrieve` | `GET /internal/v3/api/knowledgebase/wiki_publications/{publicationUuid}` | Retrieve an active Wiki provider and generation metadata. |
| `wikiPublications.routes.resolve` | `POST /internal/v3/api/knowledgebase/wiki_publications/{publicationUuid}/routes/resolve` | Resolve a normalized provider route or reviewed redirect. |
| `wikiPublications.contents.retrieve` | `GET /internal/v3/api/knowledgebase/wiki_publications/{publicationUuid}/contents/{contentHandle}` | Retrieve the exact currently public pinned representation. |
| `wikiPublications.navigation.list` | `GET /internal/v3/api/knowledgebase/wiki_publications/{publicationUuid}/navigation` | List a keyset-paginated public navigation window. |
| `wikiPublications.pages.search` | `GET /internal/v3/api/knowledgebase/wiki_publications/{publicationUuid}/pages/search` | Search a keyset-paginated public metadata projection. |

Tenant and organization scope come from the authenticated internal principal, never URL/body
selectors. Ineligible, private, paused, wrong-scope, and missing public resources use the same
non-disclosing not-found class.

The current content operation validates the opaque handle, page public version, exact Drive version,
length, and SHA-256, then returns a buffered representation capped at 16 MiB. It does not expose a
Range or streaming contract. The current search operation covers title, canonical route, and source
path metadata; it is not rendition-backed full-text search.

## Generation

Generation is owned by `tools/knowledgebase_sdk_generate.mjs` and invokes the canonical
`../sdkwork-sdk-generator/bin/sdkgen.js`. Generated transport output is limited to
`generated/server-openapi`; it must not be hand-edited. Consumers import the private composed
facade `@sdkwork/knowledgebase-internal-sdk`, which delegates to the generated client. Raw HTTP
clients, manual ingress-token headers, and local DTO copies are forbidden.

## Verification

```text
node sdks/sdkwork-knowledgebase-internal-sdk/bin/generate-sdk.mjs --check
node --test sdks/sdkwork-knowledgebase-internal-sdk/tests/sdk-family-smoke.test.mjs
pnpm --dir sdks/sdkwork-knowledgebase-internal-sdk/sdkwork-knowledgebase-internal-sdk-typescript typecheck
```
