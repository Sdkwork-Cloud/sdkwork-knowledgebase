use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    KnowledgeIdGenerator, KnowledgeIdGeneratorError, SqliteKnowledgeAgentProfileStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_agent_profile_store::KnowledgeAgentProfileStore;
use sdkwork_knowledgebase_contract::rag::{
    KnowledgeAgentBindingRequest, KnowledgeAgentProfileRequest, KnowledgeAgentStatus,
};
use sqlx::{AnyPool, Row};
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn sqlite_agent_profile_store_persists_profile_and_knowledge_bindings() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let store = SqliteKnowledgeAgentProfileStore::with_id_generator(
        pool.clone(),
        9001,
        fixed_id_generator([501, 601, 602]),
    );

    let created = store
        .create_profile(profile_request("Support Agent"))
        .await
        .unwrap();
    let first = store
        .create_binding(binding_request(created.profile_id, 7, 20, true))
        .await
        .unwrap();
    let second = store
        .create_binding(binding_request(created.profile_id, 9, 10, false))
        .await
        .unwrap();

    let loaded = store.retrieve_profile(created.profile_id).await.unwrap();

    assert_eq!(loaded.profile_id, 501);
    assert_eq!(loaded.tenant_id, 9001);
    assert_eq!(loaded.name, "Support Agent");
    assert_eq!(loaded.model_provider_id, "provider.model.openai");
    assert_eq!(loaded.model_id, "gpt-4.1");
    assert_eq!(loaded.retrieval_profile_id, Some(31));
    assert_eq!(loaded.bindings, vec![first.clone(), second.clone()]);
    assert_eq!(first.binding_id, 601);
    assert_eq!(first.space_id, 7);
    assert_eq!(first.priority, 20);
    assert!(first.enabled);
    assert!(!second.enabled);

    let row = sqlx::query(
        r#"
        SELECT tenant_id, model_provider_id, model_id, status
        FROM kb_agent_profile
        WHERE id = ?
        "#,
    )
    .bind(created.profile_id as i64)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.get::<i64, _>("tenant_id"), 9001);
    assert_eq!(
        row.get::<String, _>("model_provider_id"),
        "provider.model.openai"
    );
    assert_eq!(row.get::<String, _>("model_id"), "gpt-4.1");
    assert_eq!(row.get::<i64, _>("status"), 1);
}

#[tokio::test]
async fn sqlite_agent_profile_store_updates_and_soft_deletes_bindings() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let store = SqliteKnowledgeAgentProfileStore::with_id_generator(
        pool.clone(),
        9001,
        fixed_id_generator([501, 601]),
    );
    let created = store
        .create_profile(profile_request("Support Agent"))
        .await
        .unwrap();
    let binding = store
        .create_binding(binding_request(created.profile_id, 7, 20, true))
        .await
        .unwrap();

    let updated_profile = store
        .update_profile(
            created.profile_id,
            profile_request("Knowledge Support Agent"),
        )
        .await
        .unwrap();
    assert_eq!(updated_profile.name, "Knowledge Support Agent");

    let updated_binding = store
        .update_binding(
            created.profile_id,
            binding.binding_id,
            binding_request(created.profile_id, 8, 5, false),
        )
        .await
        .unwrap();
    assert_eq!(updated_binding.space_id, 8);
    assert_eq!(updated_binding.priority, 5);
    assert!(!updated_binding.enabled);

    store
        .delete_binding(created.profile_id, binding.binding_id)
        .await
        .unwrap();
    assert!(store
        .list_bindings(created.profile_id)
        .await
        .unwrap()
        .is_empty());

    store.delete_profile(created.profile_id).await.unwrap();
    let active_profile_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM kb_agent_profile WHERE status = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(active_profile_count, 0);
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

async fn sqlite_pool() -> AnyPool {
    sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
        "sqlite::memory:",
    )
    .await
    .unwrap()
}

async fn apply_sqlite_migration(_pool: &AnyPool) {}

fn profile_request(name: &str) -> KnowledgeAgentProfileRequest {
    KnowledgeAgentProfileRequest {
        tenant_id: 9001,
        name: name.to_string(),
        description: Some("Answers from support knowledge bases.".to_string()),
        system_instruction: "Answer with citations.".to_string(),
        model_provider_id: "provider.model.openai".to_string(),
        model_id: "gpt-4.1".to_string(),
        model_parameters: Some(r#"{"temperature":0.2}"#.to_string()),
        retrieval_profile_id: Some(31),
        citation_policy: Some(r#"{"required":true}"#.to_string()),
        memory_policy_ref: Some("memory.short_term".to_string()),
        tool_policy_ref: Some("tools.read_only".to_string()),
        answer_policy: Some(r#"{"style":"concise"}"#.to_string()),
        status: KnowledgeAgentStatus::Active,
        knowledge_mode: Default::default(),
        agent_implementation_id: sdkwork_knowledgebase_contract::default_agent_implementation_id(),
    }
}

fn binding_request(
    profile_id: u64,
    space_id: u64,
    priority: i32,
    enabled: bool,
) -> KnowledgeAgentBindingRequest {
    KnowledgeAgentBindingRequest {
        tenant_id: 9001,
        profile_id,
        space_id,
        collection_id: None,
        source_filter: None,
        document_filter: None,
        priority,
        top_k: Some(3),
        min_score: Some(0.75),
        enabled,
    }
}
