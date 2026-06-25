# Audit Investigation Runbook

Status: active  
Owner: SDKWork Knowledgebase security operators

## Data sources

- Structured logs with `audit_event = knowledge.*`
- Prometheus counters: `knowledge_audit_*`
- Durable table: `kb_audit_event`
- Framework HTTP audit: `web_audit_event` when Postgres or WEB_STORE sqlite adapters are enabled

## Framework HTTP audit query

```sql
SELECT request_id, tenant_id, user_id, api_surface, path, method, operation_id, status_code, duration_ms, created_at
FROM web_audit_event
WHERE tenant_id = $1
  AND created_at >= $2
ORDER BY created_at DESC
LIMIT 200;
```

## Production boot requirements

Production-like HTTP surfaces fail closed when no framework audit emitter is available. Before rollout:

1. Apply `database/migrations/postgres/0006_web_audit_event.up.sql` (or sqlite `0006` for standalone sqlite).
2. Set `SDKWORK_KNOWLEDGEBASE_DATABASE_URL` on app-api, backend-api, and open-api processes.
3. Alternatively for standalone sqlite dev only, configure `WEB_STORE` so `sdkwork-web-store-sqlx` can bootstrap `web_audit_event`.

## Investigation steps

1. Collect `x-request-id`, tenant id, actor id, and approximate timestamp from the reporter.
2. Query `kb_audit_event` filtered by `tenant_id`, `event_type`, and `created_at`.
3. Correlate with API access logs and IAM audit events for the same session.
4. For permission changes, inspect:
   - `knowledge.document.visibility_changed`
   - `knowledge.space.member_granted`
   - `knowledge.space.member_revoked`
   - `knowledge.backend.admin_operation`

## Example query

```sql
SELECT event_type, actor_type, actor_id, resource_type, resource_id, result, payload, created_at
FROM kb_audit_event
WHERE tenant_id = $1
  AND created_at >= $2
ORDER BY created_at DESC
LIMIT 200;
```

## Escalation

If audit rows are missing for a confirmed mutation, treat as a severity-1 logging gap and open a platform defect.

## Verification

- Replayed investigation steps reproduce the actor and resource trail.
- Post-fix mutations create new `kb_audit_event` rows.
