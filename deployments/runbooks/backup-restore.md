# Backup and restore runbook

## PostgreSQL (production)

### Backup (logical)

```bash
pg_dump --format=custom --no-owner --file=knowledgebase-$(date +%Y%m%d).dump "$SDKWORK_DATABASE_URL"
```

Store dumps in encrypted object storage with 30-day retention minimum.

### Restore (staging drill)

```bash
pg_restore --clean --if-exists --no-owner --dbname="$TARGET_DATABASE_URL" knowledgebase-YYYYMMDD.dump
pnpm db:migrate
pnpm db:status
```

Run application readiness checks against `/readyz` before traffic cutover.

## Drive object storage

Drive objects are owned by `sdkwork-drive`. Cloud profiles must back up and replicate the remote
bucket selected by `SDKWORK_KNOWLEDGEBASE_DRIVE_STORAGE_PROVIDER_ID` using provider-native controls.
Standalone profiles back up the configured local Drive provider root. Record the provider version,
bucket, encryption policy, object versioning state, and restore checkpoint with each database backup;
database and object restores must use a mutually consistent recovery point.

## Outbox webhook

Document `SDKWORK_KNOWLEDGEBASE_OUTBOX_WEBHOOK_URL` and signing secret rotation in the platform secret manager. Outbox dispatch fails closed outside `development` when webhook configuration is missing.

## Verification checklist

- [ ] `/livez` returns 200 on all API pods
- [ ] `/readyz` returns 200 when database and drive pools are healthy
- [ ] Worker processes queued ingestion jobs after restore
- [ ] Every restored API/worker replica has a healthy row in `sdkwork_node_registry`
- [ ] `/readyz` fails when the fenced Snowflake node lease cannot be renewed
- [ ] Cloud startup fails when the Drive provider is missing, inactive, local-only, or its bucket is unavailable
