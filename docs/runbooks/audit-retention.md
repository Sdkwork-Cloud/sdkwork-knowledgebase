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

Use the backend compliance API (requires `knowledge.platform.manage`, `knowledge.admin`, or `knowledge.*`):

```http
POST /backend/v3/api/knowledge/compliance/audit_events/export
Content-Type: application/json

{ "actorId": "<iam_subject_id>" }
```

- OpenAPI operation: `compliance.auditEvents.export`
- Backend SDK: `client.knowledge.compliance.auditEvents.export({ actorId })`
- Response envelope: `SdkWorkApiResponse` with `data.item.items[]` (`KnowledgeAuditEventItem`)
- Tenant scope is derived from the authenticated principal; do not pass `tenant_id` in the body.

Deliver the exported archive through the platform DPO workflow.

## GDPR delete (right to erasure)

Use the backend compliance API to anonymize actor identifiers while retaining event type and timestamps:

```http
POST /backend/v3/api/knowledge/compliance/audit_events/anonymize_actor
Content-Type: application/json

{ "actorId": "<iam_subject_id>" }
```

- OpenAPI operation: `compliance.auditEvents.anonymizeActor`
- Backend SDK: `client.knowledge.compliance.auditEvents.anonymizeActor({ actorId })`
- Response envelope: `SdkWorkApiResponse` with `data.item.anonymizedCount`
- Rows are updated to `actor_id = 'gdpr-redacted'`, `actor_type = 'system'`

Before invoking anonymization:

1. Verify legal basis and scope with platform legal.
2. Do not delete aggregate billing counters; redact PII in structured logs per log retention policy.

## Verification

- Security tests assert durable `kb_audit_event` persistence.
- `pnpm test:tenant-quota` asserts OpenAPI, SDK, and runbook alignment for quota and GDPR compliance APIs.
- Production topology uses `SDKWORK_KNOWLEDGEBASE_LOG_FORMAT=json` for log pipeline correlation with `x-request-id`.
