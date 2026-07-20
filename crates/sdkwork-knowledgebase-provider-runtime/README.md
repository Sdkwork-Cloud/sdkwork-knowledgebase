# sdkwork-knowledgebase-provider-runtime

Domain: intelligence  
Capability: knowledgebase-provider-runtime  
Package type: Rust provider runtime  
Status: active

## Public API

The crate exposes a bounded outbound HTTP runtime, Provider execution context, request/response
types, retry/circuit/bulkhead policy, stable error taxonomy, target-origin validation, and a
telemetry port. Vendor adapters own upstream wire DTOs and authentication choice; they do not own
HTTP client construction or resilience policy.

## Security

Production policy requires HTTPS and an exact configured origin. Requests reject embedded URL
credentials and fragments. Redirects are disabled, response and diagnostic bodies are bounded,
sensitive header values are never included in errors, and telemetry uses bounded identifiers.

## Configuration

Runtime policy is constructed from typed configuration. `SDKWORK_KNOWLEDGEBASE_ENVIRONMENT` selects
production HTTPS enforcement for `production` and `staging`; all other values are development/test
profiles. Provider URLs and credentials remain adapter/binding configuration, not runtime globals.

## Extension Points

Implement `ProviderTelemetry` in the Knowledgebase observability component. Do not add vendor DTOs
or Provider-specific URL logic to this crate.

## Verification

```bash
cargo test -p sdkwork-knowledgebase-provider-runtime
```

