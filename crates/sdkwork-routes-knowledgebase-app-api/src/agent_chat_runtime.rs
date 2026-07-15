use sdkwork_agent_kernel::{KnowledgeDocument, KnowledgeDocumentFilter, KnowledgeDocumentKind};
use sdkwork_intelligence_knowledgebase_repository_sqlx::SqliteKnowledgeRetrievalProfileStore;
use sdkwork_intelligence_knowledgebase_service::knowledge_engine::{
    format_scoped_document_id, parse_namespace_space_id, parse_scoped_document_id,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_engine::KnowledgeEngine;
use sdkwork_knowledgebase_agent_provider::async_bridge::block_on_async;
use sdkwork_knowledgebase_agent_provider::{
    retrieval_methods_for_strategy, KnowledgeRetrievalPlan, KnowledgeRetrievalPlanResolver,
    KnowledgeSpaceModeResolver, KnowledgebaseRetrievalClient, OkfKnowledgeClient,
    SpaceKnowledgeEngineClient,
};
use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineListRequest, KnowledgeEngineReadRequest, KnowledgeEngineSearchRequest,
};
use sdkwork_knowledgebase_contract::okf::OkfConceptSummary;
use sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode;
use sdkwork_knowledgebase_contract::rag::KnowledgeRetrievalRequest;
use std::sync::Arc;

use crate::ports::KnowledgeRetrievalAppService;
use crate::runtime::{HostedRetrievalService, KnowledgebaseRuntime};
use crate::KnowledgeAppRequestContext;

#[derive(Clone)]
pub struct RuntimeRetrievalPlanResolver {
    store: Arc<SqliteKnowledgeRetrievalProfileStore>,
}

impl RuntimeRetrievalPlanResolver {
    pub fn new(store: Arc<SqliteKnowledgeRetrievalProfileStore>) -> Self {
        Self { store }
    }
}

#[async_trait::async_trait]
impl KnowledgeRetrievalPlanResolver for RuntimeRetrievalPlanResolver {
    async fn resolve_plan(
        &self,
        tenant_id: u64,
        retrieval_profile_id: Option<u64>,
    ) -> Result<Option<KnowledgeRetrievalPlan>, String> {
        let Some(profile_id) = retrieval_profile_id else {
            return Ok(None);
        };

        let profile = self
            .store
            .get_profile(profile_id)
            .await
            .map_err(|error| error.to_string())?;
        if profile.tenant_id != tenant_id {
            return Err("retrieval profile tenant_id must match request tenant_id".to_string());
        }

        Ok(Some(KnowledgeRetrievalPlan {
            methods: retrieval_methods_for_strategy(&profile.strategy),
            top_k: Some(profile.top_k),
            min_score: profile.min_score,
        }))
    }
}

#[derive(Clone)]
pub struct RuntimeSpaceModeResolver {
    runtime: KnowledgebaseRuntime,
}

impl RuntimeSpaceModeResolver {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }
}

#[async_trait::async_trait]
impl KnowledgeSpaceModeResolver for RuntimeSpaceModeResolver {
    async fn knowledge_mode_for_space(
        &self,
        space_id: u64,
    ) -> Result<KnowledgeAgentKnowledgeMode, String> {
        let space = self
            .runtime
            .get_space_for_authorized_operation(space_id)
            .await
            .map_err(|error| error.to_string())?;
        Ok(space.knowledge_mode)
    }
}

#[derive(Clone)]
pub struct RuntimeKnowledgebaseRetrievalClient {
    runtime: KnowledgebaseRuntime,
}

impl RuntimeKnowledgebaseRetrievalClient {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }

    fn resolve_engine_for_space(
        &self,
        space_id: u64,
    ) -> Result<std::sync::Arc<dyn KnowledgeEngine>, String> {
        let runtime = self.runtime.clone();
        block_on_async(async move {
            runtime
                .knowledge_engine_space_resolver()
                .resolve_for_space(space_id, None)
                .await
                .map_err(|error| error.to_string())
        })
        .map_err(|error| error.to_string())?
    }
}

impl KnowledgebaseRetrievalClient for RuntimeKnowledgebaseRetrievalClient {
    fn retrieve(
        &self,
        request: KnowledgeRetrievalRequest,
    ) -> Result<sdkwork_knowledgebase_contract::KnowledgeRetrievalResult, String> {
        let retrieval = HostedRetrievalService::new(self.runtime.clone());
        let context = KnowledgeAppRequestContext {
            tenant_id: request.tenant_id,
            actor_id: request.actor_id,
            organization_id: None,
            session_id: None,
            request_id: uuid::Uuid::new_v4().to_string(),
            trace_id: None,
            idempotency_key: None,
        };
        block_on_async(async move { retrieval.retrieve(context, request).await })
            .map_err(|error| error.to_string())?
            .map_err(|error| error.to_string())
    }

    fn read_document(&self, document_id: &str) -> Result<KnowledgeDocument, String> {
        let (space_id, scoped_id) = if let Some(parsed) = parse_scoped_document_id(document_id) {
            parsed
        } else if let Ok(document_row_id) = document_id.parse::<u64>() {
            let runtime = self.runtime.clone();
            let document = block_on_async(async move {
                runtime
                    .document_store()
                    .get_document_by_id(document_row_id)
                    .await
            })
            .map_err(|error| error.to_string())?
            .map_err(|error| error.to_string())?;
            (document.space_id, document.id.to_string())
        } else {
            return Err(
                "document_id must use scoped form {space_id}/{document_id} or a numeric kb_document id"
                    .to_string(),
            );
        };

        let engine = self.resolve_engine_for_space(space_id)?;
        let engine_kind = engine.clone();
        let tenant_id = self.runtime.tenant_id();
        let document = block_on_async(async move {
            engine
                .read_document(KnowledgeEngineReadRequest {
                    tenant_id,
                    space_id,
                    document_id: scoped_id,
                })
                .await
        })
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;

        Ok(KnowledgeDocument::new(
            format_scoped_document_id(space_id, &document.document_id),
            map_engine_document_kind(engine_kind.as_ref()),
            document.title,
            document.content,
        )
        .with_namespace(format!("space:{space_id}"))
        .with_source_uri(document.source_uri.unwrap_or_default()))
    }

    fn list_documents(
        &self,
        filter: KnowledgeDocumentFilter,
    ) -> Result<Vec<KnowledgeDocument>, String> {
        let space_id = parse_namespace_space_id(filter.namespace.as_deref())?;
        let engine = self.resolve_engine_for_space(space_id)?;
        let engine_kind = engine.clone();
        let tenant_id = self.runtime.tenant_id();
        let listed = block_on_async(async move {
            engine
                .list_documents(KnowledgeEngineListRequest {
                    tenant_id,
                    space_id,
                    limit: 64,
                })
                .await
        })
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;

        Ok(listed
            .items
            .into_iter()
            .map(|item| {
                KnowledgeDocument::new(
                    format_scoped_document_id(space_id, &item.document_id),
                    map_engine_document_kind(engine_kind.as_ref()),
                    item.title,
                    "",
                )
                .with_namespace(format!("space:{space_id}"))
                .with_source_uri(item.source_uri.unwrap_or_default())
            })
            .filter(|document| filter.matches(document))
            .collect())
    }
}

#[derive(Clone)]
pub struct RuntimeOkfKnowledgeClient {
    runtime: KnowledgebaseRuntime,
}

impl RuntimeOkfKnowledgeClient {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }

    fn resolve_engine_for_space(
        &self,
        space_id: u64,
    ) -> Result<std::sync::Arc<dyn KnowledgeEngine>, String> {
        let runtime = self.runtime.clone();
        block_on_async(async move {
            runtime
                .knowledge_engine_space_resolver()
                .resolve_for_space(space_id, None)
                .await
                .map_err(|error| error.to_string())
        })
        .map_err(|error| error.to_string())?
    }
}

impl OkfKnowledgeClient for RuntimeOkfKnowledgeClient {
    fn search_okf_concepts(
        &self,
        space_id: u64,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<OkfConceptSummary>, String> {
        let engine = self.resolve_engine_for_space(space_id)?;
        let tenant_id = self.runtime.tenant_id();
        let query = query.to_string();
        block_on_async(async move {
            let search = engine
                .search(KnowledgeEngineSearchRequest {
                    tenant_id,
                    space_id,
                    query,
                    top_k: top_k.max(1) as u32,
                })
                .await
                .map_err(|error| error.to_string())?;

            Ok(search
                .hits
                .into_iter()
                .map(|hit| {
                    let concept_id = hit.document.document_id.clone();
                    OkfConceptSummary {
                        title: hit.document.title,
                        concept_id: concept_id.clone(),
                        concept_type: String::new(),
                        logical_path: hit
                            .document
                            .source_uri
                            .clone()
                            .unwrap_or_else(|| format!("okf/{concept_id}.md")),
                        bundle_relative_path: format!("{concept_id}.md"),
                        description: hit.snippet,
                        source_count: 0,
                        updated_at: String::new(),
                        tags: Vec::new(),
                    }
                })
                .collect())
        })
        .map_err(|error| error.to_string())?
    }

    fn read_okf_concept_content(
        &self,
        space_id: u64,
        logical_path: &str,
    ) -> Result<String, String> {
        let engine = self.resolve_engine_for_space(space_id)?;
        let tenant_id = self.runtime.tenant_id();
        let logical_path = logical_path.to_string();
        block_on_async(async move {
            let document = engine
                .read_document(KnowledgeEngineReadRequest {
                    tenant_id,
                    space_id,
                    document_id: logical_path,
                })
                .await
                .map_err(|error| error.to_string())?;

            Ok(document.content)
        })
        .map_err(|error| error.to_string())?
    }
}

fn map_engine_document_kind(engine: &dyn KnowledgeEngine) -> KnowledgeDocumentKind {
    if engine.descriptor().native {
        if engine.descriptor().implementation_id.contains("okf") {
            KnowledgeDocumentKind::Spec
        } else {
            KnowledgeDocumentKind::Other
        }
    } else {
        KnowledgeDocumentKind::ExternalReference
    }
}

#[derive(Clone)]
pub struct RuntimeSpaceKnowledgeEngineClient {
    runtime: KnowledgebaseRuntime,
}

impl RuntimeSpaceKnowledgeEngineClient {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }

    async fn resolve_engine_for_space(
        &self,
        space_id: u64,
    ) -> Result<std::sync::Arc<dyn KnowledgeEngine>, String> {
        self.runtime
            .knowledge_engine_space_resolver()
            .resolve_for_space(space_id, None)
            .await
            .map_err(|error| error.to_string())
    }
}

#[async_trait::async_trait]
impl SpaceKnowledgeEngineClient for RuntimeSpaceKnowledgeEngineClient {
    async fn search_space(
        &self,
        tenant_id: u64,
        space_id: u64,
        query: &str,
        top_k: u32,
    ) -> Result<sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineSearchResult, String>
    {
        let engine = self.resolve_engine_for_space(space_id).await?;
        engine
            .search(KnowledgeEngineSearchRequest {
                tenant_id,
                space_id,
                query: query.to_string(),
                top_k,
            })
            .await
            .map_err(|error| error.to_string())
    }

    async fn agent_provider_id_for_space(&self, space_id: u64) -> Result<String, String> {
        let engine = self.resolve_engine_for_space(space_id).await?;
        Ok(engine.descriptor().agent_provider_id)
    }

    async fn read_space_document(
        &self,
        tenant_id: u64,
        space_id: u64,
        scoped_document_id: &str,
    ) -> Result<sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineDocument, String>
    {
        let engine = self.resolve_engine_for_space(space_id).await?;
        engine
            .read_document(KnowledgeEngineReadRequest {
                tenant_id,
                space_id,
                document_id: scoped_document_id.to_string(),
            })
            .await
            .map_err(|error| error.to_string())
    }
}
