# Token Rotation Runbook

Status: active  
Owner: SDKWork Knowledgebase operators

## Scope

Rotate IAM signing keys, API keys, WeChat integration secrets, and outbox webhook signing secrets without tenant data loss.

## Preconditions

- Maintenance window approved for backend-api and worker restarts.
- `SDKWORK_KNOWLEDGEBASE_ENVIRONMENT` set for the target environment.
- Redis rate-limit store healthy when running production-like multi-replica layouts.

## Procedure

1. Rotate tenant-bound IAM signing keys through SDKWork IAM admin surfaces.
2. Update `SDKWORK_KNOWLEDGEBASE_SECRETS_ENCRYPTION_KEY` or key file and restart API + worker pods.
3. Rotate Open API keys through IAM; verify old keys return `401` and new keys succeed.
4. Rotate outbox webhook HMAC secret in runtime secret store; replay a test outbox event.
5. Confirm `/readyz` is green on app-api, backend-api, open-api, and worker `:18085/readyz`.
6. Run `pnpm test:smoke` against the deployment base URL when available.

## Rollback

Restore previous secret material from the approved secret manager version and restart affected pods.

## Verification

- Dual-token login succeeds for organization sessions on app-api.
- Backend-api rejects personal (`TENANT`) sessions.
- No spike in `knowledge_audit_*` failure logs.
