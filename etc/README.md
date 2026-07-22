# SDKWork Knowledgebase Source Configuration

`sdkwork.deployment.config.json` is the single source-controlled profile index for SDKWork
Knowledgebase. It selects typed profile values from `topology/`; the topology contract is
`../specs/topology.spec.json` and the global authority is
`../sdkwork-specs/SOURCE_CONFIG_SPEC.md`.

Supported source profiles are `standalone.development`, `standalone.production`,
`cloud.development`, and `cloud.production`. Standalone development owns the local application
gateway and worker. Cloud development starts clients only and consumes explicit deployed
application and platform surfaces.

Additional safe templates:

- `sdkwork-api-cloud-gateway.knowledgebase.development.toml`: development gateway policy.
- `sdkwork-api-cloud-gateway.knowledgebase.production.toml`: production gateway policy.
- `examples/knowledgebase-redis.env.example`: non-secret Redis configuration example.
- `examples/browser-security-headers.nginx.conf`: browser static-host security headers.

Host-local overrides such as `.env.postgres`, `.env.local`, and `etc/**/*.local.*` stay out of
source control. Passwords, ingress tokens, signing masters, certificates, and provider credentials
come from the deployment secret manager or mounted ignored secret files. Installed runtime config
is materialized to the locations governed by `../sdkwork-specs/RUNTIME_DIRECTORY_SPEC.md`; source
`etc/` is never used as mutable runtime state.

Validate this authority with:

```powershell
node ../sdkwork-specs/tools/check-source-config-standard.mjs --root .
pnpm topology:validate
pnpm deploy:validate
```
