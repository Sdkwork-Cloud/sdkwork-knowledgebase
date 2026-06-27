use sdkwork_database_id::default_snowflake_epoch_millis;
use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    KnowledgeIdGenerator, KnowledgeIdGeneratorError, SnowflakeKnowledgeIdGenerator,
    SqliteKnowledgeSpaceStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::{
    CreateKnowledgeSpaceRecord, KnowledgeSpaceStore,
};
use sqlx::AnyPool;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn sqlite_space_insert_uses_injected_runtime_snowflake_id() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let generated_id = 9_223_000_000_001_u64;
    let store = SqliteKnowledgeSpaceStore::with_id_generator(
        pool.clone(),
        9001,
        7001,
        fixed_id_generator([generated_id]),
    );

    let created = store
        .create_space(CreateKnowledgeSpaceRecord {
            name: "Snowflake Space".to_string(),
            description: None,
            okf_bundle_initialized: false,
            knowledge_mode: Default::default(),
        })
        .await
        .unwrap();

    assert_eq!(created.id, generated_id);

    let stored_id: i64 = sqlx::query_scalar("SELECT id FROM kb_space WHERE uuid = $1")
        .bind(created.uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored_id, generated_id as i64);
}

#[tokio::test]
async fn sqlite_core_tables_reject_missing_runtime_ids() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;

    let result = sqlx::query(
        r#"
        INSERT INTO kb_space (
            uuid,
            tenant_id,
            organization_id,
            name,
            status,
            okf_bundle_initialized,
            created_at,
            updated_at,
            version
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind("space-without-id")
    .bind(1_i64)
    .bind(0_i64)
    .bind("Missing Runtime ID")
    .bind(1_i64)
    .bind(0_i64)
    .bind("2026-06-05T00:00:00Z")
    .bind("2026-06-05T00:00:00Z")
    .bind(0_i64)
    .execute(&pool)
    .await;

    assert!(
        result.is_err(),
        "SQLite kb_* tables must not generate ids when runtime insert omits id"
    );
}

#[test]
fn sqlite_repository_inserts_declare_explicit_id_columns() {
    for (path, source) in [
        (
            "src/sqlite_space_stores.rs",
            include_str!("../src/sqlite_space_stores.rs"),
        ),
        (
            "src/sqlite_import_stores.rs",
            include_str!("../src/sqlite_import_stores.rs"),
        ),
        (
            "src/drive_object_ref_store.rs",
            include_str!("../src/drive_object_ref_store.rs"),
        ),
        (
            "src/okf_concept_store.rs",
            include_str!("../src/okf_concept_store.rs"),
        ),
    ] {
        for insert in kb_insert_column_blocks(source) {
            assert!(
                insert
                    .columns
                    .iter()
                    .any(|column| column.eq_ignore_ascii_case("id")),
                "{} insert into {} must bind an explicit runtime-generated id; columns: {:?}",
                path,
                insert.table_name,
                insert.columns
            );
        }
    }
}

#[test]
fn snowflake_generator_accepts_configured_node_id_and_rejects_invalid_values() {
    let generator = SnowflakeKnowledgeIdGenerator::from_node_id_config(Some("42")).unwrap();
    assert_eq!(generator.node_id(), 42);
    assert_eq!(generator.epoch_millis(), default_snowflake_epoch_millis());

    assert!(
        SnowflakeKnowledgeIdGenerator::from_node_id_config(Some("1024"))
            .unwrap_err()
            .to_string()
            .contains("exceeds max node id")
    );
    assert!(
        SnowflakeKnowledgeIdGenerator::from_node_id_config(Some("abc"))
            .expect("orchestration identifiers hash to valid node ids")
            .node_id()
            <= sdkwork_database_id::max_snowflake_node_id()
    );
    assert!(
        SnowflakeKnowledgeIdGenerator::from_node_id_config(Some("   "))
            .unwrap_err()
            .to_string()
            .contains("is required")
    );
}

#[derive(Debug)]
struct FixedIdGenerator {
    ids: Mutex<Vec<u64>>,
}

impl KnowledgeIdGenerator for FixedIdGenerator {
    fn next_id(&self) -> Result<u64, KnowledgeIdGeneratorError> {
        self.ids
            .lock()
            .expect("fixed id generator lock poisoned")
            .pop()
            .ok_or_else(|| {
                KnowledgeIdGeneratorError::Internal("fixed id generator exhausted".into())
            })
    }
}

fn fixed_id_generator(ids: impl IntoIterator<Item = u64>) -> Arc<dyn KnowledgeIdGenerator> {
    let mut ids = ids.into_iter().collect::<Vec<_>>();
    ids.reverse();
    Arc::new(FixedIdGenerator {
        ids: Mutex::new(ids),
    })
}

#[derive(Debug)]
struct InsertColumns {
    table_name: String,
    columns: Vec<String>,
}

fn kb_insert_column_blocks(source: &str) -> Vec<InsertColumns> {
    let mut inserts = Vec::new();
    let mut rest = source;
    while let Some(position) = rest.find("INSERT INTO kb_") {
        let block = &rest[position..];
        let Some(values_position) = block.find("VALUES") else {
            break;
        };
        let insert_header = &block[..values_position];
        let table_name = insert_header
            .split_whitespace()
            .nth(2)
            .expect("insert table name")
            .trim()
            .to_string();
        let columns_start = insert_header.find('(').expect("insert columns start");
        let columns_end = insert_header[columns_start + 1..]
            .find(')')
            .map(|end| columns_start + 1 + end)
            .expect("insert columns end");
        let columns = insert_header[columns_start + 1..columns_end]
            .split(',')
            .map(|column| column.trim().to_string())
            .filter(|column| !column.is_empty())
            .collect();
        inserts.push(InsertColumns {
            table_name,
            columns,
        });
        rest = &block[values_position + "VALUES".len()..];
    }
    inserts
}

async fn sqlite_pool() -> AnyPool {
    sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
        "sqlite::memory:",
    )
    .await
    .unwrap()
}

async fn apply_sqlite_migration(_pool: &AnyPool) {}
