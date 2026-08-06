use sdkwork_database_id::default_snowflake_epoch_millis;
use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    KnowledgeIdGenerator, KnowledgeIdGeneratorError, SnowflakeKnowledgeIdGenerator,
    PostgresKnowledgeSpaceStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::{
    CreateKnowledgeSpaceRecord, KnowledgeSpaceStore,
};
use sqlx::AnyPool;
use std::sync::{Arc, Mutex};



#[test]
fn postgres_repository_inserts_declare_explicit_id_columns() {
    for (path, source) in [
        (
            "src/postgres_space_stores.rs",
            include_str!("../src/postgres_space_stores.rs"),
        ),
        (
            "src/postgres_import_stores.rs",
            include_str!("../src/postgres_import_stores.rs"),
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
        SnowflakeKnowledgeIdGenerator::from_node_id_config(Some("pod-name"))
            .unwrap_err()
            .to_string()
            .contains("decimal integer")
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

