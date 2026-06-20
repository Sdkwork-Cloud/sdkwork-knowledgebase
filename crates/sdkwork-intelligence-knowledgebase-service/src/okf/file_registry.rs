use super::PersistedStandardFiles;
use crate::ports::knowledge_drive_storage::KnowledgeObjectRef;
use crate::ports::knowledge_okf_bundle_file_store::{
    CreateKnowledgeOkfBundleFileRecord, KnowledgeOkfBundleFileStore,
    KnowledgeOkfBundleFileStoreError,
};
use sdkwork_knowledgebase_contract::okf_bundle_file::{KnowledgeOkfBundleFile, OkfBundleFileKind};
use thiserror::Error;

pub struct OkfBundleFileRegistryService<'a> {
    store: &'a dyn KnowledgeOkfBundleFileStore,
}

impl<'a> OkfBundleFileRegistryService<'a> {
    pub fn new(store: &'a dyn KnowledgeOkfBundleFileStore) -> Self {
        Self { store }
    }

    pub async fn register_standard_files(
        &self,
        space_id: u64,
        files: &PersistedStandardFiles,
    ) -> Result<Vec<KnowledgeOkfBundleFile>, OkfBundleFileRegistryServiceError> {
        let mut entries = Vec::with_capacity(4);
        entries.push(
            self.register_file(space_id, &files.agents_md, OkfBundleFileKind::BundleAgents)
                .await?,
        );
        entries.push(
            self.register_file(
                space_id,
                &files.profile_yaml,
                OkfBundleFileKind::BundleProfile,
            )
            .await?,
        );
        entries.push(
            self.register_file(space_id, &files.index_md, OkfBundleFileKind::BundleIndex)
                .await?,
        );
        entries.push(
            self.register_file(space_id, &files.log_md, OkfBundleFileKind::BundleLog)
                .await?,
        );
        Ok(entries)
    }

    async fn register_file(
        &self,
        space_id: u64,
        object_ref: &KnowledgeObjectRef,
        file_kind: OkfBundleFileKind,
    ) -> Result<KnowledgeOkfBundleFile, OkfBundleFileRegistryServiceError> {
        self.store
            .create_file_entry(CreateKnowledgeOkfBundleFileRecord {
                space_id,
                logical_path: object_ref.logical_path.clone(),
                file_kind,
                artifact_role: object_ref.object_role.clone(),
                drive_bucket: object_ref.bucket.clone(),
                drive_object_key: object_ref.object_key.clone(),
                checksum_sha256_hex: object_ref.checksum_sha256_hex.clone(),
            })
            .await
            .map_err(OkfBundleFileRegistryServiceError::Store)
    }
}

#[derive(Debug, Error)]
pub enum OkfBundleFileRegistryServiceError {
    #[error(transparent)]
    Store(#[from] KnowledgeOkfBundleFileStoreError),
}
