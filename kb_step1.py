import io, re

def read(p):
    return io.open(p, 'r', encoding='utf-8').read().replace('\r\n', '\n')

def write(p, c):
    io.open(p, 'w', encoding='utf-8', newline='\n').write(c)

# ============ lib.rs ============
p = 'crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/lib.rs'
c = read(p)
# remove sqlite mod declarations
for line in ['mod sqlite_chunk_store;\n', 'mod sqlite_commerce_store;\n', 'mod sqlite_context_binding_store;\n',
             'mod sqlite_drive_import_metadata_store;\n', 'mod sqlite_group_space_binding_store;\n',
             'mod sqlite_import_stores;\n', 'mod sqlite_knowledge_document_metadata_transaction;\n',
             'mod sqlite_markdown_index_metadata_store;\n', 'mod sqlite_okf_candidate_transaction;\n',
             'mod sqlite_okf_concept_revision_metadata_store;\n', 'mod sqlite_okf_concept_transaction;\n',
             'mod sqlite_outbox_store;\n', 'mod sqlite_space_stores;\n']:
    c = c.replace(line, '')
# remove sqlite re-exports from db:: block
c = c.replace('''    connect_sqlite_and_install_schema, connect_sqlite_pool, database_config_from_url,
    install_sqlite_core_schema, install_sqlite_schema, is_postgres_database_url,
    knowledgebase_health_check, knowledgebase_process_pool_budget_from_url, postgres_health_check,
    require_postgres_rls_organization_id, require_postgres_rls_tenant_id,
    set_postgres_session_organization_id, set_postgres_session_tenant_id, sqlite_health_check,
    KnowledgebaseProcessPoolBudget, PostgresRepositoryError, POSTGRES_ORGANIZATION_SESSION_KEY,
    POSTGRES_TENANT_SESSION_KEY,''', '''    database_config_from_url, is_postgres_database_url, knowledgebase_health_check,
    knowledgebase_process_pool_budget_from_url, postgres_health_check,
    require_postgres_rls_organization_id, require_postgres_rls_tenant_id,
    set_postgres_session_organization_id, set_postgres_session_tenant_id,
    KnowledgebaseProcessPoolBudget, PostgresRepositoryError, POSTGRES_ORGANIZATION_SESSION_KEY,
    POSTGRES_TENANT_SESSION_KEY,''')
# remove pub use sqlite_* lines
for line in list(c.split('\n')):
    if re.match(r'pub use sqlite_[a-z_]+::', line.strip()):
        c = c.replace(line + '\n', '')
write(p, c)
print('lib.rs cleaned')

# ============ db/mod.rs ============
p = 'crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/db/mod.rs'
c = read(p)
print('--- db/mod.rs ---')
print(c)
