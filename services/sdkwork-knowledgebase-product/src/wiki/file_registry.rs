use super::PersistedStandardFiles;
use crate::ports::knowledge_drive_storage::KnowledgeObjectRef;
use crate::ports::knowledge_wiki_file_entry_store::{
    CreateKnowledgeWikiFileEntryRecord, KnowledgeWikiFileEntryStore,
    KnowledgeWikiFileEntryStoreError,
};
use sdkwork_knowledgebase_contract::wiki_file::{KnowledgeWikiFileEntry, WikiFileEntryType};
use thiserror::Error;

pub struct KnowledgeWikiFileRegistryService<'a> {
    store: &'a dyn KnowledgeWikiFileEntryStore,
}

impl<'a> KnowledgeWikiFileRegistryService<'a> {
    pub fn new(store: &'a dyn KnowledgeWikiFileEntryStore) -> Self {
        Self { store }
    }

    pub async fn register_standard_files(
        &self,
        space_id: u64,
        files: &PersistedStandardFiles,
    ) -> Result<Vec<KnowledgeWikiFileEntry>, KnowledgeWikiFileRegistryServiceError> {
        let mut entries = Vec::with_capacity(4);
        entries.push(
            self.register_file(space_id, &files.agents_md, WikiFileEntryType::WikiSchema)
                .await?,
        );
        entries.push(
            self.register_file(space_id, &files.schema_yaml, WikiFileEntryType::WikiSchema)
                .await?,
        );
        entries.push(
            self.register_file(space_id, &files.index_md, WikiFileEntryType::WikiIndex)
                .await?,
        );
        entries.push(
            self.register_file(space_id, &files.log_md, WikiFileEntryType::WikiLog)
                .await?,
        );
        Ok(entries)
    }

    async fn register_file(
        &self,
        space_id: u64,
        object_ref: &KnowledgeObjectRef,
        entry_type: WikiFileEntryType,
    ) -> Result<KnowledgeWikiFileEntry, KnowledgeWikiFileRegistryServiceError> {
        self.store
            .create_file_entry(CreateKnowledgeWikiFileEntryRecord {
                space_id,
                logical_path: object_ref.logical_path.clone(),
                entry_type,
                artifact_role: object_ref.object_role.clone(),
                drive_bucket: object_ref.bucket.clone(),
                drive_object_key: object_ref.object_key.clone(),
                checksum_sha256_hex: object_ref.checksum_sha256_hex.clone(),
            })
            .await
            .map_err(KnowledgeWikiFileRegistryServiceError::Store)
    }
}

#[derive(Debug, Error)]
pub enum KnowledgeWikiFileRegistryServiceError {
    #[error(transparent)]
    Store(#[from] KnowledgeWikiFileEntryStoreError),
}
