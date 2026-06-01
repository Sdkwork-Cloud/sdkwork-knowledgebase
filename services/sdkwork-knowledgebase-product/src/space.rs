use crate::ports::knowledge_space_store::{
    CreateKnowledgeSpaceRecord, KnowledgeSpaceStore, KnowledgeSpaceStoreError,
};
use crate::wiki::{KnowledgeWikiInitializerService, KnowledgeWikiInitializerServiceError};
use sdkwork_knowledgebase_contract::space::{CreateKnowledgeSpaceRequest, KnowledgeSpace};
use thiserror::Error;

pub struct KnowledgeSpaceService<'a> {
    store: &'a dyn KnowledgeSpaceStore,
    wiki_initializer: &'a KnowledgeWikiInitializerService<'a>,
}

impl<'a> KnowledgeSpaceService<'a> {
    pub fn new(
        store: &'a dyn KnowledgeSpaceStore,
        wiki_initializer: &'a KnowledgeWikiInitializerService<'a>,
    ) -> Self {
        Self {
            store,
            wiki_initializer,
        }
    }

    pub async fn create_space(
        &self,
        request: CreateKnowledgeSpaceRequest,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceServiceError> {
        if request.name.trim().is_empty() {
            return Err(KnowledgeSpaceServiceError::InvalidRequest(
                "name is required".to_string(),
            ));
        }

        let space = self
            .store
            .create_space(CreateKnowledgeSpaceRecord {
                name: request.name,
                description: request.description,
                llm_wiki_initialized: false,
            })
            .await
            .map_err(KnowledgeSpaceServiceError::Store)?;

        self.wiki_initializer
            .initialize_standard_files(space.id, &space.name)
            .await?;

        self.store
            .mark_llm_wiki_initialized(space.id)
            .await
            .map_err(KnowledgeSpaceServiceError::Store)
    }
}

#[derive(Debug, Error)]
pub enum KnowledgeSpaceServiceError {
    #[error("invalid knowledge space request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Store(#[from] KnowledgeSpaceStoreError),
    #[error(transparent)]
    WikiInitializer(#[from] KnowledgeWikiInitializerServiceError),
}
