use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::{
    ports::{
        knowledge_access_control::KnowledgeAccessRole,
        knowledge_site_store::{
            CreateKnowledgeSiteHostBindingRecord, KnowledgeSiteStore, KnowledgeSiteStoreError,
        },
    },
    site::{KnowledgeSitePublicationService, KnowledgeSitePublicationServiceError},
};
use sdkwork_knowledgebase_contract::{
    CreateKnowledgeSiteHostBindingRequest, KnowledgeSite, KnowledgeSiteHostBinding,
    KnowledgeSiteHostBindingState, KnowledgeSiteHostBindingType, KnowledgeSitePublicationResult,
    KnowledgeSiteRelease, PublishKnowledgeSiteReleaseRequest, RollbackKnowledgeSiteReleaseRequest,
    UpsertKnowledgeSiteRequest,
};
use sdkwork_utils_rust::SdkWorkPageData;

use crate::{
    hosted_access::{ensure_runtime_tenant, require_actor_id, require_space_access_with_role},
    runtime::KnowledgebaseRuntime,
    ApiError, ApiResult, KnowledgeAppRequestContext, KnowledgeSiteAppService,
};

#[derive(Clone)]
pub(crate) struct HostedSiteService {
    runtime: KnowledgebaseRuntime,
}

impl HostedSiteService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }

    fn publication_service<'a>(
        &'a self,
        operator_id: &'a str,
    ) -> KnowledgeSitePublicationService<'a> {
        KnowledgeSitePublicationService::new(
            self.runtime.tenant_id(),
            self.runtime.organization_id(),
            operator_id,
            self.runtime.site_store(),
            self.runtime.space_store(),
            self.runtime.okf_concept_store(),
            self.runtime.drive_storage(),
            self.runtime.site_artifact_store(),
        )
    }

    async fn require_site_access(
        &self,
        context: &KnowledgeAppRequestContext,
        site_id: u64,
        role: KnowledgeAccessRole,
    ) -> ApiResult<KnowledgeSite> {
        let site = self
            .runtime
            .site_store()
            .get_site(site_id)
            .await
            .map_err(map_store_error)?;
        require_space_access_with_role(&self.runtime, context, site.space_id, role).await?;
        Ok(site)
    }
}

#[async_trait]
impl KnowledgeSiteAppService for HostedSiteService {
    async fn retrieve_site(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
    ) -> ApiResult<KnowledgeSite> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        require_space_access_with_role(
            &self.runtime,
            &context,
            space_id,
            KnowledgeAccessRole::Reader,
        )
        .await?;
        self.runtime
            .site_store()
            .get_site_by_space(space_id)
            .await
            .map_err(map_store_error)
    }

    async fn upsert_site(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
        request: UpsertKnowledgeSiteRequest,
    ) -> ApiResult<KnowledgeSite> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        if request.space_id != space_id {
            return Err(ApiError::invalid_request(
                "invalid_site_request",
                "request spaceId must match the route spaceId",
            ));
        }
        require_space_access_with_role(
            &self.runtime,
            &context,
            space_id,
            KnowledgeAccessRole::Writer,
        )
        .await?;
        let operator_id = require_actor_id(&context)?;
        self.publication_service(&operator_id)
            .upsert_site(request)
            .await
            .map_err(map_publication_error)
    }

    async fn publish_site_release(
        &self,
        context: KnowledgeAppRequestContext,
        site_id: u64,
        request: PublishKnowledgeSiteReleaseRequest,
    ) -> ApiResult<KnowledgeSitePublicationResult> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        self.require_site_access(&context, site_id, KnowledgeAccessRole::Writer)
            .await?;
        let operator_id = require_actor_id(&context)?;
        self.publication_service(&operator_id)
            .publish(
                site_id,
                request.expected_site_version,
                &standalone_public_base_url(),
            )
            .await
            .map_err(map_publication_error)
    }

    async fn list_site_releases(
        &self,
        context: KnowledgeAppRequestContext,
        site_id: u64,
        cursor: Option<String>,
        page_size: Option<u32>,
    ) -> ApiResult<SdkWorkPageData<KnowledgeSiteRelease>> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        self.require_site_access(&context, site_id, KnowledgeAccessRole::Reader)
            .await?;
        let page_size = crate::pagination::normalize_api_page_size(page_size)?;
        let cursor = crate::pagination::parse_u64_cursor(cursor.as_deref()).map_err(|_| {
            ApiError::invalid_request("invalid_parameter", "cursor must be a valid release id")
        })?;
        let (items, next_cursor, has_more) = self
            .runtime
            .site_store()
            .list_releases_page(site_id, cursor, page_size)
            .await
            .map_err(map_store_error)?;
        Ok(crate::pagination::cursor_page_data(
            items,
            next_cursor.map(|value| value.to_string()),
            has_more,
            page_size,
        ))
    }

    async fn retrieve_site_release(
        &self,
        context: KnowledgeAppRequestContext,
        release_id: u64,
    ) -> ApiResult<KnowledgeSiteRelease> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        let release = self
            .runtime
            .site_store()
            .get_release(release_id)
            .await
            .map_err(map_store_error)?;
        self.require_site_access(&context, release.site_id, KnowledgeAccessRole::Reader)
            .await?;
        Ok(release)
    }

    async fn rollback_site_release(
        &self,
        context: KnowledgeAppRequestContext,
        site_id: u64,
        request: RollbackKnowledgeSiteReleaseRequest,
    ) -> ApiResult<KnowledgeSite> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        self.require_site_access(&context, site_id, KnowledgeAccessRole::Writer)
            .await?;
        let operator_id = require_actor_id(&context)?;
        self.publication_service(&operator_id)
            .rollback(site_id, request)
            .await
            .map_err(map_publication_error)
    }

    async fn list_site_host_bindings(
        &self,
        context: KnowledgeAppRequestContext,
        site_id: u64,
        cursor: Option<String>,
        page_size: Option<u32>,
    ) -> ApiResult<SdkWorkPageData<KnowledgeSiteHostBinding>> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        self.require_site_access(&context, site_id, KnowledgeAccessRole::Reader)
            .await?;
        let page_size = crate::pagination::normalize_api_page_size(page_size)?;
        let cursor = crate::pagination::parse_u64_cursor(cursor.as_deref()).map_err(|_| {
            ApiError::invalid_request(
                "invalid_parameter",
                "cursor must be a valid host binding id",
            )
        })?;
        let (items, next_cursor, has_more) = self
            .runtime
            .site_store()
            .list_host_bindings_page(site_id, cursor, page_size)
            .await
            .map_err(map_store_error)?;
        Ok(crate::pagination::cursor_page_data(
            items,
            next_cursor.map(|value| value.to_string()),
            has_more,
            page_size,
        ))
    }

    async fn create_site_host_binding(
        &self,
        context: KnowledgeAppRequestContext,
        site_id: u64,
        request: CreateKnowledgeSiteHostBindingRequest,
    ) -> ApiResult<KnowledgeSiteHostBinding> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        self.require_site_access(&context, site_id, KnowledgeAccessRole::Owner)
            .await?;
        let (normalized_host, lifecycle_state) = normalize_requested_host(
            request.binding_type,
            &request.host,
        )?;
        self.runtime
            .site_store()
            .create_host_binding(CreateKnowledgeSiteHostBindingRecord {
                site_id,
                binding_type: request.binding_type,
                normalized_host,
                canonical: request.canonical
                    && request.binding_type == KnowledgeSiteHostBindingType::CustomPrefix,
                lifecycle_state,
                web_server_site_id: None,
                web_server_domain_id: None,
                web_server_deployment_id: None,
                expected_site_version: request.expected_site_version,
            })
            .await
            .map_err(map_store_error)
    }

    async fn delete_site_host_binding(
        &self,
        context: KnowledgeAppRequestContext,
        site_id: u64,
        binding_id: u64,
        expected_site_version: u64,
    ) -> ApiResult<()> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        self.require_site_access(&context, site_id, KnowledgeAccessRole::Owner)
            .await?;
        self.runtime
            .site_store()
            .delete_host_binding(site_id, binding_id, expected_site_version)
            .await
            .map_err(map_store_error)
    }
}

fn normalize_requested_host(
    binding_type: KnowledgeSiteHostBindingType,
    host: &str,
) -> ApiResult<(String, KnowledgeSiteHostBindingState)> {
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
    match binding_type {
        KnowledgeSiteHostBindingType::SystemId => Err(ApiError::invalid_request(
            "invalid_site_host_binding",
            "system ID host bindings are created by the site service",
        )),
        KnowledgeSiteHostBindingType::CustomPrefix => {
            if host.is_empty()
                || host.len() > 63
                || host.starts_with('-')
                || host.ends_with('-')
                || !host
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            {
                return Err(ApiError::invalid_request(
                    "invalid_site_host_binding",
                    "custom prefix must be one lowercase DNS label",
                ));
            }
            Ok((
                format!("{host}.kb.sdkwork.com"),
                KnowledgeSiteHostBindingState::Active,
            ))
        }
        KnowledgeSiteHostBindingType::ExternalDomain => {
            if host.is_empty() || !host.contains('.') {
                return Err(ApiError::invalid_request(
                    "invalid_site_host_binding",
                    "external domain must be a fully qualified DNS hostname",
                ));
            }
            Ok((host, KnowledgeSiteHostBindingState::Pending))
        }
    }
}

fn standalone_public_base_url() -> String {
    std::env::var("SDKWORK_KNOWLEDGEBASE_PUBLIC_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:18081".to_string())
}

fn map_publication_error(error: KnowledgeSitePublicationServiceError) -> ApiError {
    match error {
        KnowledgeSitePublicationServiceError::InvalidRequest(detail) => {
            ApiError::invalid_request("invalid_site_publication_request", detail)
        }
        KnowledgeSitePublicationServiceError::NotFound => {
            ApiError::not_found("site_publication_not_found", "site publication not found")
        }
        KnowledgeSitePublicationServiceError::VersionConflict => ApiError::conflict(
            "site_publication_version_conflict",
            "site version changed; reload before retrying",
        ),
        KnowledgeSitePublicationServiceError::Storage(_) => ApiError::new(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "site_publication_storage_unavailable",
            "site publication storage is unavailable",
        ),
        KnowledgeSitePublicationServiceError::Internal(_) => {
            ApiError::internal("site_publication_internal", "site publication failed")
        }
    }
}

fn map_store_error(error: KnowledgeSiteStoreError) -> ApiError {
    match error {
        KnowledgeSiteStoreError::InvalidRequest(detail) => {
            ApiError::invalid_request("invalid_site_request", detail)
        }
        KnowledgeSiteStoreError::NotFound => {
            ApiError::not_found("site_not_found", "site resource not found")
        }
        KnowledgeSiteStoreError::VersionConflict => ApiError::conflict(
            "site_version_conflict",
            "site version changed; reload before retrying",
        ),
        KnowledgeSiteStoreError::Conflict(detail) => {
            ApiError::conflict("site_conflict", detail)
        }
        KnowledgeSiteStoreError::Internal(_) => {
            ApiError::internal("site_store_internal", "site persistence failed")
        }
    }
}

