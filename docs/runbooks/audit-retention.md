# Audit Event Retention and GDPR Operations

Status: active  
Owner: SDKWork Knowledgebase operators  
Related: [tenant-isolation.md](tenant-isolation.md), [backup-restore.md](../../deployments/runbooks/backup-restore.md)

## Scope

Tables:

- `kb_audit_event` — domain audit trail (visibility, members, admin mutations)
- `web_audit_event` — framework HTTP audit persistence

## Retention policy (default)

| Environment | Retention | Action |
| --- | --- | --- |
| Production | 365 days | Scheduled purge job |
| Staging | 90 days | Scheduled purge job |
| Development | 30 days | Manual or cron |

Configure via operator job; canonical SQL lives under `database/operations/` when Phase 2.4 ships.

## GDPR export (tenant data subject request)

1. Identify `tenant_id` and subject `actor_id` from IAM.
2. Export matching rows:
   ```sql
   SELECT * FROM kb_audit_event
   WHERE tenant_id = $1 AND actor_id = $2
   ORDER BY created_at;
   ```
3. Deliver encrypted archive to platform DPO workflow.

## GDPR delete (right to erasure)

1. Verify legal basis and scope with platform legal.
2. Anonymize actor identifiers in `kb_audit_event` for the subject (retain event type/timestamp for compliance).
3. Do not delete aggregate billing counters; redact PII in structured logs per log retention policy.

## Verification

- Security tests assert durable `kb_audit_event` persistence.
- Production topology uses `SDKWORK_KNOWLEDGEBASE_LOG_FORMAT=json` for log pipeline correlation with `x-request-id`.
