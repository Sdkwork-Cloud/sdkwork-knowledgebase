# Backup and restore runbook

## PostgreSQL (production)

### Backup (logical)

```bash
pg_dump --format=custom --no-owner --file=knowledgebase-$(date +%Y%m%d).dump "$SDKWORK_KNOWLEDGEBASE_DATABASE_URL"
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

Drive objects are owned by `sdkwork-drive`. Back up the configured drive storage root or remote bucket using provider-native snapshot/replication policies aligned with RPO/RTO targets.

## Outbox webhook

Document `SDKWORK_KNOWLEDGEBASE_OUTBOX_WEBHOOK_URL` and signing secret rotation in the platform secret manager. Outbox dispatch fails closed outside `development` when webhook configuration is missing.

## Verification checklist

- [ ] `/livez` returns 200 on all API pods
- [ ] `/readyz` returns 200 when database and drive pools are healthy
- [ ] Worker processes queued ingestion jobs after restore
- [ ] Snowflake node IDs are unique per pod (`SDKWORK_KNOWLEDGEBASE_SNOWFLAKE_NODE_ID`)
