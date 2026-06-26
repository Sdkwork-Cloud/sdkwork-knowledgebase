# SDKWork Knowledgebase — deployment artifacts

Production deployment descriptors for `cloud.split-services.production` topology.

## Contents

| Path | Purpose |
|------|---------|
| `docker/Dockerfile.api` | App / Backend / Open API server image (single binary via `KB_API_BINARY`) |
| `docker/Dockerfile.worker` | Background worker (outbox + ingestion maintenance) |
| `kubernetes/app-api-deployment.yaml` | App API Deployment + Service |
| `kubernetes/backend-api-deployment.yaml` | Backend API Deployment + Service |
| `kubernetes/open-api-deployment.yaml` | Open API Deployment + Service |
| `kubernetes/worker-deployment.yaml` | Background worker Deployment |
| `kubernetes/ingress.yaml` | NGINX Ingress for app/backend/open API paths |
| `kubernetes/hpa.yaml` | HorizontalPodAutoscaler for API and worker Deployments |
| `kubernetes/poddisruptionbudget.yaml` | PodDisruptionBudget for rolling update safety |
| `kubernetes/networkpolicy.yaml` | Restrict ingress to NGINX and monitoring namespaces |
| `kubernetes/servicemonitor.yaml` | Prometheus Operator scrape targets for `/metrics` |
| `runbooks/backup-restore.md` | PostgreSQL and Drive object backup/restore |
| `runbooks/production-launch.md` | Production cutover sequencing, smoke gates, and rollback |

## Quick start (Kubernetes)

1. Build and push images (replace registry):
   ```bash
   docker build -f deployments/docker/Dockerfile.api -t registry.sdkwork.com/apps/sdkwork-knowledgebase/api:0.1.0 .
   docker build -f deployments/docker/Dockerfile.worker -t registry.sdkwork.com/apps/sdkwork-knowledgebase/worker:0.1.0 .
   ```
2. Apply secrets and config from `configs/topology/cloud.split-services.production.env`.
3. Apply manifests:
   ```bash
   kubectl apply -f deployments/kubernetes/
   ```
4. Verify probes:
   - Liveness: `GET /livez`
   - Readiness: `GET /readyz`

## Observability

| Variable | Purpose |
|----------|---------|
| `RUST_LOG` | Tracing filter (e.g. `info,sdkwork_knowledgebase_api_server=debug`) |
| `SDKWORK_KNOWLEDGEBASE_LOG_FORMAT` | Set to `json` for structured JSON logs in production aggregators |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | When set, API/worker processes export traces over OTLP/HTTP (requires `otel` feature build) |
| `OTEL_SERVICE_NAME` | Overrides the default OpenTelemetry service name per process |

HTTP APIs emit an `x-request-id` response header (or echo inbound `x-request-id`) for request correlation. Prometheus metrics are exposed at `GET /metrics` on API and worker health processes, including `knowledgebase_health_status` (updated by `/readyz`). **Do not expose `/metrics` on public ingress**; use in-cluster ServiceMonitor scraping only.

Structured audit events (for example `knowledge.document.visibility_changed`, `knowledge.space.member_granted`, `knowledge.space.member_revoked`, `okf.concept.published`) are written to structured logs with an `audit_event` field. Related Prometheus counters are exported at `GET /metrics` (`knowledge_audit_*`).

Billable usage counters (`knowledge_retrievals_total`, `knowledge_context_packs_total`, `knowledge_ingest_jobs_succeeded_total`, `knowledge_ingest_jobs_failed_total`) and structured `billing_event` JSON log lines support commercial metering pipelines.

Post-deploy smoke check (optional):

```bash
SDKWORK_KNOWLEDGEBASE_SMOKE_BASE_URL=https://knowledgebase.sdkwork.com pnpm test:smoke
```

Optional PC renderer shell probe (requires a running Vite preview or dev server):

```bash
SDKWORK_KNOWLEDGEBASE_E2E_BASE_URL=http://127.0.0.1:5173 pnpm test:e2e
```

Playwright shell smoke (build + preview + Chromium):

```bash
pnpm --dir apps/sdkwork-knowledgebase-pc run test:e2e:install
pnpm test:e2e:playwright
```

## Tenant isolation

Each API/worker process is bound to a single runtime tenant via `SDKWORK_KNOWLEDGEBASE_TENANT_ID`. Authenticated request context must match that tenant; mismatches return `403` with `tenant_id_mismatch` (fail-closed). Optional `SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID` enforces the same pattern for organization scope.

Multi-tenant SaaS deployments should run **one process (or dedicated database schema) per tenant**, not share a single runtime across tenants. Tenant isolation is enforced at the application layer (SQL `tenant_id` filters plus runtime guards); PostgreSQL row-level security is not required for the current single-tenant-per-process model.

Integration coverage: `crates/sdkwork-routes-knowledgebase-app-api/tests/integration_tenant_isolation.rs`.

## Backend authorization

Backend API operations require the `knowledge.admin` permission (or `knowledge.*`) on the authenticated operator's access token. Mutations are audited as `knowledge.backend.admin_operation` structured log events and exported via `knowledge_audit_backend_admin_operation_total` at `GET /metrics`.

## Related specs

- `../sdkwork-specs/DEPLOYMENT_SPEC.md`
- `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_SPEC.md`
- `../specs/topology.spec.json`

Status: active.
