# External Knowledge Engine Catalog

SDKWork Knowledgebase uses this directory to **register** well-known open-source knowledge/RAG platforms and vector stores that can be integrated through the [Knowledge Engine SPI](../../specs/knowledge-engine-spi.spec.json).

This catalog is **metadata-first**. Upstream source code is pinned optionally via git submodules under `upstream/{vendorId}/`.

## Layout

```text
external/knowledge-engines/
├── README.md                 # this file
├── catalog.manifest.json     # authoritative vendor index
├── vendors/
│   └── {vendorId}/
│       └── engine.manifest.json
└── upstream/
    └── {vendorId}/           # optional git submodule pin (see .gitmodules)
```

## Submodule policy

1. **Catalog manifest** records every supported external engine (`catalog.manifest.json` + `vendors/*/engine.manifest.json`).
2. **Git submodule** is added only when an adapter team needs upstream source locally (compliance review, API diff, contract tests).
3. Submodule path convention: `external/knowledge-engines/upstream/{vendorId}`.
4. Register submodule URLs in `.gitmodules` using:

```bash
git submodule add -b main https://github.com/{org}/{repo}.git external/knowledge-engines/upstream/{vendorId}
```

Or dry-run planning:

```bash
node tools/sync_external_knowledge_engine_submodules.mjs --check
```

## Integration tiers

| Tier | Meaning |
|------|---------|
| `catalog` | Registered in catalog; no SDKWork adapter yet |
| `stub` | Adapter crate skeleton + health/search contract tests |
| `adapter` | Partial SPI (search/read/list/health) implemented |
| `production` | Supported for space binding and agent provider registration |

## Normative specs

- `specs/external-knowledge-engine-catalog.spec.json`
- `specs/knowledge-engine-spi.spec.json`

## Verification

```bash
node tools/check_external_knowledge_engine_catalog.mjs
```
