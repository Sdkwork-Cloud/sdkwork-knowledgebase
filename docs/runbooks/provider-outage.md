# Provider Outage Runbook

Status: active  
Owner: SDKWork Knowledgebase operators

## Scope

External dependency failures: embedding provider, external knowledge engines, Drive storage, IAM database, Redis rate-limit store, webhook delivery targets.

## Detection

- Elevated `5xx` on retrieval or ingest endpoints.
- Worker logs show repeated outbox publish failures.
- `/readyz` failures on database or Drive pool checks.

## Response

1. Identify failing dependency from structured logs and `x-request-id` trace correlation.
2. If Redis is unavailable in production-like environments, API startup should fail closed; restore Redis before scaling replicas.
3. For engine outages, disable the affected source/profile in backend-api and drain ingestion jobs.
4. For webhook outages, verify outbox retry metrics; pause destructive replays if downstream is unhealthy.
5. Communicate user impact: search may degrade to keyword-only when embeddings are unavailable.

## Recovery

1. Restore dependency health.
2. Re-enable sources/profiles.
3. Requeue stale outbox events if required.
4. Run smoke retrieval and a sample ingest job.

## Verification

- P95 retrieval latency returns within PRD target (< 2s warm index).
- Outbox pending queue drains.
