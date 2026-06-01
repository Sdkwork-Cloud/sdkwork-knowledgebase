mod file_registry;
mod index_renderer;
mod initializer;
mod log_renderer;
mod schema_renderer;

use crate::ports::knowledge_drive_storage::{
    KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError, PutKnowledgeObjectRequest,
};
use sdkwork_knowledgebase_contract::wiki::{LlmWikiPaths, WikiLogEntry, WikiPageSummary};

pub use file_registry::{KnowledgeWikiFileRegistryService, KnowledgeWikiFileRegistryServiceError};
pub use index_renderer::render_index_md;
pub use initializer::{KnowledgeWikiInitializerService, KnowledgeWikiInitializerServiceError};
pub use log_renderer::render_log_md;
pub use schema_renderer::{render_agents_md, render_wiki_schema_yaml};

#[derive(Debug, Clone)]
pub struct PersistStandardFilesRequest {
    pub space_name: String,
    pub pages: Vec<WikiPageSummary>,
    pub log_entries: Vec<WikiLogEntry>,
}

#[derive(Debug, Clone)]
pub struct PersistedStandardFiles {
    pub agents_md: KnowledgeObjectRef,
    pub schema_yaml: KnowledgeObjectRef,
    pub index_md: KnowledgeObjectRef,
    pub log_md: KnowledgeObjectRef,
}

pub struct LlmWikiStandardFileService<'a> {
    drive: &'a dyn KnowledgeDriveStorage,
}

impl<'a> LlmWikiStandardFileService<'a> {
    pub fn new(drive: &'a dyn KnowledgeDriveStorage) -> Self {
        Self { drive }
    }

    pub async fn persist_standard_files(
        &self,
        request: PersistStandardFilesRequest,
    ) -> Result<PersistedStandardFiles, KnowledgeStorageError> {
        let paths = LlmWikiPaths::default();
        let agents_md = self
            .drive
            .put_object(PutKnowledgeObjectRequest::text(
                paths.agents_md,
                "wiki_schema",
                render_agents_md(&request.space_name),
                None,
            ))
            .await?;
        let schema_yaml = self
            .drive
            .put_object(PutKnowledgeObjectRequest::text(
                paths.schema_yaml,
                "wiki_schema",
                render_wiki_schema_yaml(),
                None,
            ))
            .await?;
        let index_md = self
            .drive
            .put_object(PutKnowledgeObjectRequest::text(
                paths.index_md,
                "wiki_index",
                render_index_md(&request.space_name, &request.pages),
                None,
            ))
            .await?;
        let log_md = self
            .drive
            .put_object(PutKnowledgeObjectRequest::text(
                paths.log_md,
                "wiki_log",
                render_log_md(&request.log_entries),
                None,
            ))
            .await?;

        Ok(PersistedStandardFiles {
            agents_md,
            schema_yaml,
            index_md,
            log_md,
        })
    }
}
