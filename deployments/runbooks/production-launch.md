# Production launch runbook

Status: active  
Application: sdkwork-knowledgebase  
Topology: `cloud.split-services.production`  
Updated: 2026-06-24

## Purpose

Operational checklist for launching SDKWork Knowledgebase in a tenant-scoped production deployment. Run this after Phase 0.1 verification passes locally and in CI.

## Pre-flight

1. Confirm secrets are provisioned outside topology files:
   - Database password (`SDKWORK_CLAW_DATABASE_PASSWORD_FILE`)
   - Secrets encryption key (`SDKWORK_KNOWLEDGEBASE_SECRETS_ENCRYPTION_KEY_FILE`)
   - Outbox webhook URL and signing secret
   - Unique `SDKWORK_KNOWLEDGEBASE_SNOWFLAKE_NODE_ID` per pod/process
2. Apply topology from `configs/topology/cloud.split-services.production.env`.
3. Ensure public ingress exposes app/backend/open API paths only. **Do not expose `/metrics` on ingress**; Prometheus scrapes in-cluster via ServiceMonitor.
4. Run repository gates:
   ```bash
   pnpm verify
   pnpm test:launch-readiness
   pnpm lint
   ```

## Database bootstrap

1. Point `SDKWORK_KNOWLEDGEBASE_DATABASE_URL` at the production PostgreSQL instance.
2. Bootstrap and validate:
   ```bash
   pnpm db:bootstrap
   pnpm db:validate
   pnpm db:drift:check
   ```
3. Record migration status output in the change ticket.

## Deployment smoke

1. Deploy Kubernetes manifests from `deployments/kubernetes/`.
2. Verify health on each API surface:
   ```bash
   SDKWORK_KNOWLEDGEBASE_SMOKE_BASE_URL=https://knowledgebase.example.com pnpm test:smoke
   ```
   Public smoke checks only probe `/livez` and `/readyz`; `/metrics` remains in-cluster only.
   For split-services deployments, probe each process independently:
   ```bash
   SDKWORK_KNOWLEDGEBASE_SMOKE_APP_URL=https://knowledgebase.example.com \
   SDKWORK_KNOWLEDGEBASE_SMOKE_BACKEND_URL=https://knowledgebase-admin.example.com \
   SDKWORK_KNOWLEDGEBASE_SMOKE_OPEN_URL=https://knowledge.example.com \
   SDKWORK_KNOWLEDGEBASE_SMOKE_WORKER_URL=https://knowledgebase-worker.example.com \
   pnpm test:smoke
   ```
3. Optional internal metrics smoke, run only from an in-cluster network path:
   ```bash
   SDKWORK_KNOWLEDGEBASE_SMOKE_METRICS_URLS=http://sdkwork-knowledgebase-app-api,http://sdkwork-knowledgebase-backend-api,http://sdkwork-knowledgebase-open-api,http://sdkwork-knowledgebase-worker:18085 \
   pnpm test:smoke
   ```
4. Optional authenticated admin probe:
   ```bash
   SDKWORK_KNOWLEDGEBASE_SMOKE_BACKEND_URL=https://knowledgebase-admin.example.com \
   SDKWORK_KNOWLEDGEBASE_SMOKE_ACCESS_TOKEN=... \
   pnpm test:launch-readiness
   ```
5. Optional integrator open API probe:
   ```bash
   SDKWORK_KNOWLEDGEBASE_SMOKE_OPEN_URL=https://knowledge.example.com \
   SDKWORK_KNOWLEDGEBASE_SMOKE_API_KEY=... \
   pnpm test:launch-readiness
   ```
6. Confirm worker `/readyz` returns 200 and queued ingestion jobs drain after a test ingest.

## Observability

1. Set `SDKWORK_KNOWLEDGEBASE_LOG_FORMAT=json` in production topology for structured log aggregation.
2. When an OTLP collector is available, set:
   - `OTEL_EXPORTER_OTLP_ENDPOINT`
   - `OTEL_SERVICE_NAME` per process (`sdkwork-knowledgebase-app-api`, `sdkwork-knowledgebase-backend-api`, `sdkwork-knowledgebase-open-api`, `sdkwork-knowledgebase-worker`)
3. Confirm ServiceMonitor targets scrape `/metrics` in-cluster and dashboards include:
   - `knowledgebase_health_status`
   - `knowledge_api_requests_total`
   - `knowledge_api_auth_failures_total`

## Release artifacts

1. Keep the browser bundle package (`web-universal-cloud-browser-zip`) disabled while its manifest
   package metadata remains `releaseStatus: prelaunch-artifact-pending`.
2. Do not publish or enable `web-universal-cloud-browser-zip` until release evidence has been
   recorded for checksum, signature, SBOM, provenance, attestation, workflow run, rollout,
   rollback, and live smoke results per `sdkwork.app.config.json`.
3. Publish or approve private-registry consumption for all three SDK families:
   - `sdkwork-knowledgebase-app-sdk`
   - `sdkwork-knowledgebase-backend-sdk`
   - `sdkwork-knowledgebase-sdk`
4. Keep desktop bundles disabled until desktop CI packaging targets ship (`releaseStatus: prelaunch-disabled`).
5. Run Playwright launch flows in CI:
   ```bash
   pnpm test:e2e:playwright
   ```

## Backup drill

Before traffic cutover, exercise `deployments/runbooks/backup-restore.md` in staging:

1. Take a logical PostgreSQL dump.
2. Restore to an isolated database.
3. Re-run `pnpm db:migrate`, `pnpm db:status`, and `/readyz` checks.

## Rollback

1. Roll back Deployments to the previous image tag.
2. Restore database only when schema drift requires it; otherwise prefer forward-fix migrations.
3. Disable ingress traffic and verify `/readyz` on the rolled-back revision before re-enabling traffic.

## Sign-off

| Gate | Owner | Evidence |
|------|-------|----------|
| Security (`pnpm test:security`) | Platform | CI run URL |
| Architecture (`pnpm check`) | App team | CI run URL |
| DB bootstrap/drift | DBA | `pnpm db:status` output |
| API smoke | SRE | `pnpm test:smoke` output |
| PC author/search E2E | Frontend | Playwright CI job |
| Release artifacts | Release | Workflow run + checksum + signature + SBOM + provenance + attestation + rollout/rollback + live smoke record |
