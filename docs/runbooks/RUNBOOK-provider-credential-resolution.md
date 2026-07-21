# Provider Credential Resolution Runbook

Status: active prelaunch procedure; production backend pending  
Owner: SDKWork Knowledgebase operators  
Decision: `ADR-20260720`

## Scope

Configure, validate, rotate, revoke, and diagnose write-only credential references used by external
Knowledge Engine Provider Bindings. This procedure never stores a plaintext credential in the
Knowledgebase database, app manifest, source `etc/`, command arguments, logs, screenshots, or
release evidence.

## Environment Policy

| Environment | Accepted locator | Required control |
| --- | --- | --- |
| `development`, `test` | `env://SDKWORK_KNOWLEDGEBASE_PROVIDER_SECRET_<PROVIDER_CODE>_<NAME>` | Exact uppercase, implementation-bound namespace |
| `development`, `test` | `file://<absolute-path>` | Canonical regular file below `SDKWORK_KNOWLEDGEBASE_PROVIDER_SECRETS_DIR/<provider-code>/` |
| `staging`, `production` | `secret://knowledgebase/provider/<provider-code>/<name>` | Injected approved `SecretProvider`; env/file always rejected |

`<provider-code>` is the suffix of `engine.knowledge.external.<provider-code>`. Hyphens are changed
to underscores and letters are uppercased only for environment-variable names. A Dify credential
cannot reference a RAGFlow variable, file, or managed secret path. Cross-Provider locators fail
closed before a secret is loaded.

## Development And Test

Environment example, showing names only:

```powershell
$env:SDKWORK_KNOWLEDGEBASE_ENVIRONMENT = 'development'
$env:SDKWORK_KNOWLEDGEBASE_PROVIDER_SECRET_DIFY_PRIMARY = '<inject-locally>'
```

Persist only this locator through the protected Provider management UI or composed backend SDK:

```text
env://SDKWORK_KNOWLEDGEBASE_PROVIDER_SECRET_DIFY_PRIMARY
```

For mounted files, create a private root outside source control and expose the root through runtime
configuration:

```text
<private-root>/
  dify/
    primary
  ragflow/
    primary
```

Set `SDKWORK_KNOWLEDGEBASE_PROVIDER_SECRETS_DIR` to the absolute `<private-root>` and construct the
locator from the absolute file path. The resolver checks lexical containment, then canonicalizes
both the per-Provider root and target. Traversal, directories, empty files, files larger than 64
KiB, non-UTF-8 content, and symlinks resolving outside the per-Provider root are rejected.

Never commit the root, secret files, local environment overlays, or the resulting locator. Do not
reuse this local policy in staging or production.

## Staging And Production

Default `KnowledgebaseRuntime::connect` deliberately fails before database startup in staging and
production. The process composition root must construct an approved managed resolver and inject it
through `KnowledgebaseRuntime::connect_with_provider_credential_resolver`.

The repository provides the typed integration boundary, immutable access context, managed locator
grammar, 64 KiB response bound, one five-second maximum budget across bulkhead admission and backend
execution, a 32-call default concurrency bulkhead, intermediate plaintext cleanup, sanitized errors,
Provider audit-record requirement, and no-cache semantics. It does not yet provide approved
production Vault/KMS evidence.
Do not enable publication or classify the app as production-ready until all of the following exist:

- A reviewed Vault/KMS/secret-manager adapter with TLS verification and explicit connect/request
  timeouts inside the backend client, not only the outer five-second wait.
- Tenant- and Provider-bound least-privilege policy using the supplied tenant, organization, space,
  Binding, credential-reference, implementation, actor, operation, trace, and deadline context.
- Durable access-denial/success audit retention, alerting, provider health, outage behavior, and
  capacity evidence.
- Executed rotation, revocation, backend outage, recovery, and rollback drills against the immutable
  release candidate.

The generic Kernel `VaultSecretProvider` and `ChainedSecretProvider` are not accepted production
evidence by themselves. Their transport TLS, internal timeout, policy, audit, and drill properties
must be reviewed and proven before use.

## Rotation

1. Create or rotate the secret in the approved external secret manager without printing the value.
2. If the secret id changes, use the Provider credential-reference rotate command with the current
   expected version and the new implementation-bound locator.
3. Test the affected draft/testing Binding through the protected management surface.
4. Verify a successful secret-manager audit record and the sanitized
   `knowledge.provider_credential.access` event correlated by trace and Binding.
5. Revoke the predecessor in the secret manager only after the observation window and rollback
   decision complete.

There is no process credential cache. The next authorized operation resolves the current reference
and observes rotation immediately.

## Revocation

1. Revoke the Knowledgebase credential reference using its current expected version.
2. Revoke the secret-manager credential according to the provider-specific procedure.
3. Confirm subsequent Binding test/execution fails closed with a sanitized unavailable or denied
   error and performs no Provider HTTP request with an old credential.
4. Investigate unexpected success as a release-blocking incident; do not restore a stale locator.

## Observability And Data Safety

Sanitized access events may contain environment, tenant, organization, space, Binding,
credential-reference version, implementation, actor, operation, trace, and outcome. They must not
contain the locator, managed secret id, secret value, authorization header, audit-record id, or raw
backend error. Metrics must use bounded Provider/operation/outcome labels and must not use tenant,
actor, Binding, credential-reference, or trace ids as labels.

The outcome is one fixed value: `granted`, `invalid_reference`, `access_denied`, `unavailable`,
`response_too_large`, or `internal`. Alert on sustained `unavailable`/`internal` and any bulkhead
saturation without adding backend error text or resource identifiers to metric labels.

## Failure Handling

- Invalid reference: verify environment policy, implementation code, URI grammar, and per-Provider
  root. Never weaken validation to accept a legacy locator.
- Access denied: verify the authenticated tenant/space/Binding context and managed policy. Do not
  retry with another Provider's secret.
- Unavailable: check the secret-manager health and bounded request deadline without logging its
  response body or locator.
- Response too large: replace the malformed secret object; do not increase the 64 KiB ceiling.
- Timeout or bulkhead saturation: investigate the backend client's own connect/request timeout and
  thread utilization. The outer timeout cannot cancel a synchronous backend call, so that call keeps
  its bulkhead permit until it returns. New calls remain bounded and fail within their total budget,
  but repeated saturation is a backend outage and release-blocking operational condition.

## Verification

```bash
cargo test -p sdkwork-knowledgebase-provider-secret-adapter
cargo clippy -p sdkwork-knowledgebase-provider-secret-adapter --all-targets -- -D warnings
cargo test -p sdkwork-intelligence-knowledgebase-service --test knowledge_engine_space_resolver
cargo test -p sdkwork-routes-knowledgebase-app-api --test hosted_runtime_routes hosted_backend_provider_management_is_scoped_versioned_and_secret_safe
pnpm check:knowledge-engine-spi
node ../sdkwork-specs/tools/check-component-port-bindings.mjs --root .
node ../sdkwork-specs/tools/check-application-layering.mjs --root .
node ../sdkwork-specs/tools/check-rust-backend-composition.mjs --root .
```

Passing local verification proves the contract boundary only. Production closure requires the live
backend, audit retention, drills, security/privacy approval, release PostgreSQL evidence, live
Provider certification, and release governance recorded in the commercialization plan.
