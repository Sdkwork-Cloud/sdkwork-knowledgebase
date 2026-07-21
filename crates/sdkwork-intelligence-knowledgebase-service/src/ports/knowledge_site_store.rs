use async_trait::async_trait;
use sdkwork_knowledgebase_contract::{
    KnowledgeSite, KnowledgeSiteHostBinding, KnowledgeSiteHostBindingState,
    KnowledgeSiteHostBindingType, KnowledgeSitePublishMode, KnowledgeSiteRelease,
    KnowledgeSiteState, KnowledgeSiteVisibility,
};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeSiteStoreError {
    #[error("invalid site request: {0}")]
    InvalidRequest(String),
    #[error("site resource not found")]
    NotFound,
    #[error("site resource version conflict")]
    VersionConflict,
    #[error("site resource conflict: {0}")]
    Conflict(String),
    #[error("site store internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpsertKnowledgeSiteRecord {
    pub space_id: u64,
    pub title: String,
    pub visibility: KnowledgeSiteVisibility,
    pub homepage_concept_id: Option<String>,
    pub theme_id: String,
    pub publish_mode: KnowledgeSitePublishMode,
    pub expected_version: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateKnowledgeSiteReleaseRecord {
    pub site_id: u64,
    pub source_content_hash: String,
    pub previous_release_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteKnowledgeSiteReleaseRecord {
    pub release_id: u64,
    pub manifest_drive_uri: String,
    pub manifest_drive_space_id: String,
    pub manifest_drive_node_id: String,
    pub manifest_checksum_sha256_hex: String,
    pub page_count: u32,
    pub asset_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateKnowledgeSiteHostBindingRecord {
    pub site_id: u64,
    pub binding_type: KnowledgeSiteHostBindingType,
    pub normalized_host: String,
    pub canonical: bool,
    pub lifecycle_state: KnowledgeSiteHostBindingState,
    pub web_server_site_id: Option<String>,
    pub web_server_domain_id: Option<String>,
    pub web_server_deployment_id: Option<String>,
    pub expected_site_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPublicKnowledgeSite {
    pub site: KnowledgeSite,
    pub release: KnowledgeSiteRelease,
    pub canonical_host: Option<String>,
}

#[async_trait]
pub trait KnowledgeSiteStore: Send + Sync {
    async fn upsert_site(
        &self,
        record: UpsertKnowledgeSiteRecord,
    ) -> Result<KnowledgeSite, KnowledgeSiteStoreError>;

    async fn get_site_by_space(
        &self,
        space_id: u64,
    ) -> Result<KnowledgeSite, KnowledgeSiteStoreError>;

    async fn get_site(&self, site_id: u64) -> Result<KnowledgeSite, KnowledgeSiteStoreError>;

    async fn create_release(
        &self,
        record: CreateKnowledgeSiteReleaseRecord,
    ) -> Result<KnowledgeSiteRelease, KnowledgeSiteStoreError>;

    async fn complete_release(
        &self,
        record: CompleteKnowledgeSiteReleaseRecord,
    ) -> Result<KnowledgeSiteRelease, KnowledgeSiteStoreError>;

    async fn fail_release(
        &self,
        release_id: u64,
        error_code: String,
    ) -> Result<KnowledgeSiteRelease, KnowledgeSiteStoreError>;

    async fn get_release(
        &self,
        release_id: u64,
    ) -> Result<KnowledgeSiteRelease, KnowledgeSiteStoreError>;

    async fn list_releases_page(
        &self,
        site_id: u64,
        cursor: Option<u64>,
        page_size: u32,
    ) -> Result<(Vec<KnowledgeSiteRelease>, Option<u64>, bool), KnowledgeSiteStoreError>;

    async fn activate_release(
        &self,
        site_id: u64,
        release_id: u64,
        expected_site_version: u64,
    ) -> Result<KnowledgeSite, KnowledgeSiteStoreError>;

    async fn create_host_binding(
        &self,
        record: CreateKnowledgeSiteHostBindingRecord,
    ) -> Result<KnowledgeSiteHostBinding, KnowledgeSiteStoreError>;

    async fn list_host_bindings_page(
        &self,
        site_id: u64,
        cursor: Option<u64>,
        page_size: u32,
    ) -> Result<(Vec<KnowledgeSiteHostBinding>, Option<u64>, bool), KnowledgeSiteStoreError>;

    async fn delete_host_binding(
        &self,
        site_id: u64,
        binding_id: u64,
        expected_site_version: u64,
    ) -> Result<(), KnowledgeSiteStoreError>;

    async fn resolve_public_site_by_space(
        &self,
        space_id: u64,
    ) -> Result<ResolvedPublicKnowledgeSite, KnowledgeSiteStoreError>;

    async fn resolve_public_site_by_host(
        &self,
        normalized_host: &str,
    ) -> Result<ResolvedPublicKnowledgeSite, KnowledgeSiteStoreError>;
}

pub fn site_is_publicly_resolvable(site: &KnowledgeSite) -> bool {
    site.lifecycle_state == KnowledgeSiteState::Active
        && matches!(
            site.visibility,
            KnowledgeSiteVisibility::Public | KnowledgeSiteVisibility::Unlisted
        )
        && site.current_release_id.is_some()
}

