# migrations/postgres

Pre-launch the knowledgebase schema is consolidated on the single greenfield
baseline: `database/ddl/baseline/postgres/0001_knowledgebase_baseline.sql`.
All post-baseline migrations (group knowledge spaces, ingestion leases,
provider bindings, live wiki publication, organization isolation, outbox
claim fencing/retry backoff, audit scope indexes) are folded into the
baseline. No ordered post-baseline migrations exist while the app is
pre-launch; shared development schemas converge by resetting the module
state to the baseline.
