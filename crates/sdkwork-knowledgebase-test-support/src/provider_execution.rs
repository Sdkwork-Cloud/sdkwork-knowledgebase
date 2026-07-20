use sdkwork_knowledgebase_contract::provider_binding::{
    KnowledgeEngineDataScope, KnowledgeEngineExecutionContext,
};

pub fn provider_execution_context(
    tenant_id: u64,
    organization_id: u64,
    space_id: u64,
    binding_id: u64,
    trace_id: impl Into<String>,
) -> KnowledgeEngineExecutionContext {
    knowledge_execution_context(
        tenant_id,
        organization_id,
        space_id,
        Some(binding_id),
        trace_id,
    )
}

pub fn knowledge_execution_context(
    tenant_id: u64,
    organization_id: u64,
    space_id: u64,
    binding_id: Option<u64>,
    trace_id: impl Into<String>,
) -> KnowledgeEngineExecutionContext {
    let now_ms = sdkwork_utils_rust::to_unix_millis(sdkwork_utils_rust::now());
    let deadline_unix_ms = u64::try_from(now_ms)
        .expect("test clock must be after the Unix epoch")
        .checked_add(60_000)
        .expect("test deadline must fit u64");
    KnowledgeEngineExecutionContext {
        tenant_id,
        organization_id,
        actor_id: "provider-test-actor".to_string(),
        permission_scope: vec!["knowledge.read".to_string()],
        data_scope: KnowledgeEngineDataScope {
            allowed_space_ids: vec![space_id],
            allowed_source_ids: Vec::new(),
            allowed_document_ids: Vec::new(),
        },
        space_id,
        binding_id,
        trace_id: trace_id.into(),
        deadline_unix_ms,
    }
}
