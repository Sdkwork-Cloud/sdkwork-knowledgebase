# Rate Limit Incident Runbook

Status: active  
Owner: SDKWork Knowledgebase operators

## Scope

Abuse, accidental retry storms, or ineffective rate limiting across replicas.

## Symptoms

- `429` spikes on auth, ingest, or mutation endpoints.
- Uneven throttling across pods (indicates in-memory fallback).

## Immediate actions

1. Confirm `SDKWORK_KNOWLEDGEBASE_REDIS_URL` or `SDKWORK_KNOWLEDGEBASE_REDIS_ENABLED` is configured in production-like environments.
2. Verify all API replicas connect to the same Redis logical database.
3. Inspect gateway and service logs for the offending client/session id.
4. Block abusive API keys or sessions through IAM admin tools.

## Hardening

- Never run multi-replica production without Redis rate-limit store.
- Keep rate-limit configuration in topology env templates under `etc/topology/`.

## Verification

- Repeated abusive requests receive consistent `429` regardless of pod.
- Normal tenant traffic recovers after unblock.
