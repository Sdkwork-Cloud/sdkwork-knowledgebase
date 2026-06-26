> Migrated from `docs/topology-standard.md` on 2026-06-24.
> Owner: SDKWork maintainers

This repository adopts the shared SDKWork runtime topology framework.

- Platform standard: `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_SPEC.md`
- Naming authority: `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_NAMING.md`
- Adoption guide: `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_ADOPTION.md`
- Framework: `../sdkwork-app-topology`

## Archetype

`application-http-gateway`: Knowledgebase exposes application HTTP surfaces through `sdkwork-routes-knowledgebase-*` route binaries. Shared IAM and appbase SDKs use `platform.api-gateway` unless a standalone profile embeds the required platform adapter.

## Default Dev Profile

`standalone.unified-process.development`

The default browser and desktop development commands use PostgreSQL, `unified-process`, and `standalone`:

```bash
pnpm dev:browser
pnpm dev:desktop
pnpm topology:validate
```

Explicit development variants use suffixed commands such as:

```bash
pnpm dev:browser:sqlite
pnpm dev:browser:postgres:split-services:cloud
```

## Local URLs

| Surface | URL |
| --- | --- |
| `application.public-ingress` | http://127.0.0.1:18081 |
| `application.backend-http` | http://127.0.0.1:18081 |
| `application.open-http` | http://127.0.0.1:18081 |
| `platform.api-gateway` | http://127.0.0.1:3900 |

Client env keys:

- `VITE_SDKWORK_KNOWLEDGEBASE_DEPLOYMENT_PROFILE`: browser-visible deployment profile.
- `VITE_SDKWORK_KNOWLEDGEBASE_APPLICATION_PUBLIC_HTTP_URL`: app SDK surface.
- `VITE_SDKWORK_KNOWLEDGEBASE_APPLICATION_BACKEND_HTTP_URL`: backend SDK surface for approved backend-admin contexts.
- `VITE_SDKWORK_KNOWLEDGEBASE_APPLICATION_OPEN_HTTP_URL`: open SDK surface.
- `VITE_SDKWORK_KNOWLEDGEBASE_PLATFORM_API_GATEWAY_HTTP_URL`: platform and IAM SDK surface.
- `VITE_SDKWORK_APPBASE_APP_API_BASE_URL`: appbase IAM app API surface.

Profile values live in `configs/topology/*.env` only. Do not hardcode ports in route crates or feature packages.

Cloud gateway config bundles:

- `configs/sdkwork-api-cloud-gateway.knowledgebase.development.toml`
- `configs/sdkwork-api-cloud-gateway.knowledgebase.production.toml`

