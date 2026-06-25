# Configs

Purpose: source-controlled safe config templates, schemas, profile examples, and non-secret defaults for SDKWork Knowledgebase.

Owner: SDKWork Knowledgebase maintainers.

## Layout

| Path | Purpose |
|------|---------|
| `topology/` | Deployment profile env templates (`standalone.*`, `cloud.*`) |
| `sdkwork-api-cloud-gateway.*.toml` | Cloud gateway bundle inputs |

Development database credentials and IAM signing secrets are **not** stored in topology templates. Copy `.env.postgres.example` to `.env.postgres` at the repository root and load it via `pnpm dev --dev-env-file .env.postgres`.

## Allowed content

- Checked-in config schemas, examples, and non-secret defaults
- Topology profile env files without live credentials

## Forbidden content

- `.local` overrides, live credentials, access tokens, private keys
- Runtime user config, database files, and Redis data

## Related specs

- `../sdkwork-specs/CONFIG_SPEC.md`
- `../sdkwork-specs/ENVIRONMENT_SPEC.md`
- `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_SPEC.md`
- `../sdkwork-specs/RUNTIME_DIRECTORY_SPEC.md`

## Verification

```bash
pnpm topology:validate
node ../sdkwork-specs/tools/check-database-framework-standard.mjs --root .
```

Status: active.
