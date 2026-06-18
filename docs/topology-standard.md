# SDKWork Knowledgebase Runtime Topology

This repository adopts the shared SDKWork runtime topology framework.

- Platform standard: `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_SPEC.md`
- Naming authority: `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_NAMING.md`
- Adoption guide: `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_ADOPTION.md`
- Framework: `../sdkwork-app-topology`

## Archetype

`application-http-gateway` — Knowledgebase exposes three **application** HTTP surfaces through `sdkwork-router-knowledgebase-*` route binaries. Shared IAM and appbase SDKs use **platform.api-gateway**.

## Default dev profile

`self-hosted.split-services.development`

## Commands

```bash
pnpm knowledgebase:dev          # self-hosted split-services development
pnpm knowledgebase:dev:cloud    # cloud-hosted split-services development
pnpm topology:validate          # validate specs/topology.spec.json
```

## Local URLs (self-hosted split dev)

| Surface | URL |
| --- | --- |
| `application.public-ingress` | http://127.0.0.1:18081 |
| `application.backend-http` | http://127.0.0.1:18082 |
| `application.open-http` | http://127.0.0.1:18083 |
| `platform.api-gateway` | http://127.0.0.1:3900 |

Client env keys:

- `VITE_SDKWORK_KNOWLEDGEBASE_APPLICATION_PUBLIC_HTTP_URL` — app SDK (`/app/v3/api`)
- `VITE_SDKWORK_KNOWLEDGEBASE_APPLICATION_BACKEND_HTTP_URL` — backend SDK (`/backend/v3/api`)
- `VITE_SDKWORK_KNOWLEDGEBASE_APPLICATION_OPEN_HTTP_URL` — open SDK (`/knowledge/v3/api`)
- `VITE_SDKWORK_KNOWLEDGEBASE_PLATFORM_API_GATEWAY_HTTP_URL` — platform / IAM SDKs
- `VITE_SDKWORK_APPBASE_APP_API_BASE_URL` — appbase IAM app API

Profile values live in `configs/topology/*.env` only. Do not hardcode ports in route crates or feature packages.

Cloud gateway config bundles (for `cloud-hosted` profiles):

- `configs/sdkwork-api-gateway.knowledgebase.development.toml`
- `configs/sdkwork-api-gateway.knowledgebase.production.toml`
