use sdkwork_agent_kernel::{KnowledgeDocument, KnowledgeDocumentFilter};
use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    SqliteKnowledgeRetrievalProfileStore, SqliteKnowledgeSpaceStore, SqliteKnowledgeWikiPageStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::{
    knowledge_drive_storage::{HeadKnowledgeObjectRequest, KnowledgeDriveStorage},
    knowledge_space_store::KnowledgeSpaceStore,
    knowledge_wiki_page_store::KnowledgeWikiPageStore,
};
use sdkwork_knowledgebase_agent_provider::{
    retrieval_methods_for_strategy, KnowledgeRetrievalPlan, KnowledgeRetrievalPlanResolver,
    KnowledgeSpaceModeResolver, KnowledgebaseRetrievalClient, LlmWikiKnowledgeClient,
};
use sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode;
use sdkwork_knowledgebase_contract::{rag::KnowledgeRetrievalRequest, wiki::WikiPageSummary};
use sdkwork_knowledgebase_drive::KnowledgebaseDriveStorageAdapter;
use std::sync::Arc;

use crate::runtime::KnowledgebaseRuntime;

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
    store: Arc<SqliteKnowledgeSpaceStore>,
}

impl RuntimeSpaceModeResolver {
    pub fn new(store: Arc<SqliteKnowledgeSpaceStore>) -> Self {
        Self { store }
    }
}

#[async_trait::async_trait]
impl KnowledgeSpaceModeResolver for RuntimeSpaceModeResolver {
    async fn knowledge_mode_for_space(
        &self,
        space_id: u64,
    ) -> Result<KnowledgeAgentKnowledgeMode, String> {
        let space = self
            .store
            .get_space(space_id)
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
}

impl KnowledgebaseRetrievalClient for RuntimeKnowledgebaseRetrievalClient {
    fn retrieve(
        &self,
        request: KnowledgeRetrievalRequest,
    ) -> Result<sdkwork_knowledgebase_contract::KnowledgeRetrievalResult, String> {
        let service = self.runtime.retrieval_service();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(service.retrieve(request))
        })
        .map_err(|error| error.to_string())
    }

    fn read_document(&self, _document_id: &str) -> Result<KnowledgeDocument, String> {
        Err("knowledgebase document read is not implemented in hosted runtime".to_string())
    }

    fn list_documents(
        &self,
        _filter: KnowledgeDocumentFilter,
    ) -> Result<Vec<KnowledgeDocument>, String> {
        Ok(Vec::new())
    }
}

#[derive(Clone)]
pub struct RuntimeLlmWikiKnowledgeClient {
    wiki_pages: Arc<SqliteKnowledgeWikiPageStore>,
    drive: Arc<KnowledgebaseDriveStorageAdapter>,
}

impl RuntimeLlmWikiKnowledgeClient {
    pub fn new(
        wiki_pages: Arc<SqliteKnowledgeWikiPageStore>,
        drive: Arc<KnowledgebaseDriveStorageAdapter>,
    ) -> Self {
        Self { wiki_pages, drive }
    }
}

impl LlmWikiKnowledgeClient for RuntimeLlmWikiKnowledgeClient {
    fn search_wiki_pages(
        &self,
        space_id: u64,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<WikiPageSummary>, String> {
        let pages = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(self.wiki_pages.list_page_summaries(space_id))
        })
        .map_err(|error| error.to_string())?;

        let normalized_query = normalize_query(query);
        let mut ranked = pages
            .into_iter()
            .map(|page| (rank_wiki_page(&page, &normalized_query), page))
            .filter(|(score, _)| *score > 0.0 || normalized_query.is_empty())
            .collect::<Vec<_>>();

        ranked.sort_by(|left, right| {
            right
                .0
                .partial_cmp(&left.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(ranked
            .into_iter()
            .take(top_k.max(1))
            .map(|(_, page)| page)
            .collect())
    }

    fn read_wiki_page_content(&self, _space_id: u64, logical_path: &str) -> Result<String, String> {
        let drive = self.drive.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let object_ref = drive
                    .head_object(HeadKnowledgeObjectRequest::managed_artifact(
                        logical_path,
                        "wiki_page",
                    ))
                    .await
                    .map_err(|error| error.to_string())?;
                drive
                    .get_object_text(&object_ref)
                    .await
                    .map_err(|error| error.to_string())
            })
        })
    }
}

fn normalize_query(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect()
}

fn rank_wiki_page(page: &WikiPageSummary, tokens: &[String]) -> f64 {
    if tokens.is_empty() {
        return 0.5;
    }

    let haystack = format!(
        "{} {} {}",
        page.title.to_lowercase(),
        page.summary.to_lowercase(),
        page.tags.join(" ").to_lowercase()
    );

    let matches = tokens
        .iter()
        .filter(|token| haystack.contains(token.as_str()))
        .count();

    matches as f64 / tokens.len() as f64
}
