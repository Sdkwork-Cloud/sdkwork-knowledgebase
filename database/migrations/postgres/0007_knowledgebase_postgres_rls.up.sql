-- Phase 2.1: Postgres RLS tenant isolation (ADR-2026-06-24-phase2-postgres-rls-multi-tenant)
-- source: crates/sdkwork-intelligence-knowledgebase-repository-sqlx/migrations/postgres/V202606260001__knowledgebase_postgres_rls.sql

-- Session key set at connection checkout: app.current_tenant_id (see postgres_tenant_session.rs)

DO $$
DECLARE
    table_name text;
BEGIN
    FOR table_name IN
        SELECT unnest(ARRAY[
            'kb_space',
            'kb_collection',
            'kb_source',
            'kb_drive_object_ref',
            'kb_document',
            'kb_document_version',
            'kb_chunk',
            'kb_index',
            'kb_embedding',
            'kb_retrieval_profile',
            'kb_retrieval_trace',
            'kb_retrieval_hit',
            'kb_agent_profile',
            'kb_agent_knowledge_binding',
            'kb_ingestion_job',
            'kb_ingestion_job_item',
            'kb_okf_concept',
            'kb_okf_concept_revision',
            'kb_okf_bundle_file',
            'kb_okf_schema_profile',
            'kb_okf_log_entry',
            'kb_local_mirror_package',
            'kb_space_context_binding',
            'kb_outbox_event',
            'kb_okf_concept_link',
            'kb_okf_candidate',
            'kb_market_listing',
            'kb_market_subscription',
            'kb_site_deployment',
            'kb_audit_event'
        ])
    LOOP
        EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', table_name);
        EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', table_name);
        EXECUTE format(
            'CREATE POLICY tenant_isolation ON %I AS PERMISSIVE FOR ALL TO PUBLIC USING (tenant_id = current_setting(''app.current_tenant_id'', true)::bigint) WITH CHECK (tenant_id = current_setting(''app.current_tenant_id'', true)::bigint)',
            table_name
        );
    END LOOP;
END $$;

-- Framework HTTP audit rows (tenant_id stored as text)
ALTER TABLE web_audit_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE web_audit_event FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON web_audit_event
    AS PERMISSIVE
    FOR ALL
    TO PUBLIC
    USING (tenant_id IS NOT DISTINCT FROM current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id IS NOT DISTINCT FROM current_setting('app.current_tenant_id', true));
