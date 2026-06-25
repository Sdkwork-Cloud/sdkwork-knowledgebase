# Jobs

Purpose: scheduled maintenance, queue consumers, and operational job descriptors for SDKWork Knowledgebase.

Owner: SDKWork Knowledgebase maintainers.

## Active jobs

| Job | Binary / entry | Topology process id | Description |
|-----|----------------|---------------------|-------------|
| Outbox + ingestion maintenance | `sdkwork-knowledgebase-worker` | `application.background-worker` | Claims pending outbox events and processes queued ingestion jobs |

## Orchestration

Production cloud topology (`cloud.split-services.production`) requires the background worker as a dedicated Deployment. See `deployments/kubernetes/worker-deployment.yaml`.

Environment variables:

- `SDKWORK_KNOWLEDGEBASE_WORKER_POLL_INTERVAL_MS` (default `5000`)
- `SDKWORK_KNOWLEDGEBASE_WORKER_OUTBOX_BATCH_SIZE` (default `50`)
- `SDKWORK_KNOWLEDGEBASE_WORKER_INGESTION_JOB_BATCH_SIZE` (default `25`)

Status: active.
