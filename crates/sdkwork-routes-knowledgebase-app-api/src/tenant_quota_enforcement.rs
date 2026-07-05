//! Tenant business quota enforcement at the app-api boundary.

use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_node_tree::{
    GetKnowledgeDriveNodeRequest, KnowledgeDriveNodeTree,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage,
};
use sdkwork_intelligence_knowledgebase_service::tenant_quota::{
    build_quota_status, ensure_document_capacity, ensure_ingest_concurrency, ensure_storage_capacity,
    TenantQuotaExceeded, TenantQuotaKind,
};
use sdkwork_knowledgebase_contract::ingest::KnowledgeDriveImportRequest;
use sdkwork_knowledgebase_contract::KnowledgeTenantQuotaStatus;
use sdkwork_knowledgebase_observability::KnowledgebaseTenantQuotaLimits;
use sdkwork_routes_knowledgebase_backend_api::knowledgebase_rate_limit_store;
use sdkwork_utils_rust::is_blank;
use sdkwork_web_core::RateLimitStore;
use std::time::Duration;

use crate::{runtime::KnowledgebaseRuntime, ApiError, ApiResult};

pub(crate) fn map_tenant_quota_error(error: TenantQuotaExceeded) -> ApiError {
    let detail = match error.kind {
        TenantQuotaKind::Documents => format!(
            "tenant document quota exceeded ({}/{})",
            error.usage, error.limit
        ),
        TenantQuotaKind::IngestConcurrency => format!(
            "tenant ingest concurrency quota exceeded ({}/{})",
            error.usage, error.limit
        ),
        TenantQuotaKind::RetrievalRate => format!(
            "tenant retrieval rate quota exceeded ({}/{})",
            error.usage, error.limit
        ),
        TenantQuotaKind::StorageBytes => format!(
            "tenant storage quota exceeded ({}/{})",
            error.usage, error.limit
        ),
    };
    ApiError::quota_exceeded(detail)
}

async fn load_storage_bytes_used(runtime: &KnowledgebaseRuntime) -> ApiResult<u64> {
    runtime
        .object_ref_store()
        .sum_active_storage_bytes()
        .await
        .map_err(|error| ApiError::internal("knowledgebase_store_failed", error.to_string()))
}

pub(crate) async fn load_tenant_quota_status(
    runtime: &KnowledgebaseRuntime,
) -> ApiResult<KnowledgeTenantQuotaStatus> {
    let limits = KnowledgebaseTenantQuotaLimits::from_env();
    let summary = runtime
        .space_store()
        .summarize_tenant_knowledgebase()
        .await
        .map_err(|error| ApiError::internal("knowledgebase_store_failed", error.to_string()))?;
    let inflight = runtime
        .ingestion_job_store()
        .count_inflight_jobs()
        .await
        .map_err(|error| ApiError::internal("knowledgebase_store_failed", error.to_string()))?;
    let storage_bytes_used = load_storage_bytes_used(runtime).await?;
    Ok(build_quota_status(
        limits,
        summary.document_count,
        inflight,
        storage_bytes_used,
    ))
}

pub(crate) async fn ensure_tenant_can_add_storage(
    runtime: &KnowledgebaseRuntime,
    additional_bytes: u64,
) -> ApiResult<()> {
    let limits = KnowledgebaseTenantQuotaLimits::from_env();
    let storage_bytes_used = load_storage_bytes_used(runtime).await?;
    ensure_storage_capacity(storage_bytes_used, additional_bytes, &limits)
        .map_err(map_tenant_quota_error)
}

async fn peek_drive_import_object_size_bytes(
    runtime: &KnowledgebaseRuntime,
    request: &KnowledgeDriveImportRequest,
) -> ApiResult<u64> {
    if let (Some(drive_space_id), Some(drive_node_id)) = (
        request
            .drive_space_id
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty()),
        request
            .drive_node_id
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty()),
    ) {
        if let Ok(Some(node)) = runtime
            .drive_tree()
            .get_node(GetKnowledgeDriveNodeRequest {
                drive_space_id: drive_space_id.to_string(),
                drive_node_id: drive_node_id.to_string(),
            })
            .await
        {
            if let Some(size_bytes) = node.size_bytes {
                return Ok(size_bytes);
            }
        }
    }

    if is_blank(Some(request.drive_storage_provider_id.as_str()))
        || is_blank(Some(request.drive_bucket.as_str()))
        || is_blank(Some(request.drive_object_key.as_str()))
    {
        return Err(ApiError::invalid_request(
            "drive_import_storage_size_unavailable",
            "cannot evaluate tenant storage quota before drive import without a resolvable object locator"
                .to_string(),
        ));
    }

    let object_ref = runtime
        .drive_storage()
        .head_object(HeadKnowledgeObjectRequest::original_document(
            request.drive_storage_provider_id.clone(),
            request.drive_bucket.clone(),
            request.drive_object_key.clone(),
        ))
        .await
        .map_err(|error| ApiError::internal("knowledge_drive_storage_failed", error.to_string()))?;

    Ok(object_ref.size_bytes)
}

pub(crate) async fn ensure_tenant_can_import_drive_object(
    runtime: &KnowledgebaseRuntime,
    request: &KnowledgeDriveImportRequest,
) -> ApiResult<()> {
    let additional_bytes = peek_drive_import_object_size_bytes(runtime, request).await?;
    ensure_tenant_can_add_storage(runtime, additional_bytes).await
}

pub(crate) async fn ensure_tenant_can_create_document(
    runtime: &KnowledgebaseRuntime,
) -> ApiResult<()> {
    let limits = KnowledgebaseTenantQuotaLimits::from_env();
    let summary = runtime
        .space_store()
        .summarize_tenant_knowledgebase()
        .await
        .map_err(|error| ApiError::internal("knowledgebase_store_failed", error.to_string()))?;
    ensure_document_capacity(summary.document_count, &limits).map_err(map_tenant_quota_error)
}

pub(crate) async fn ensure_tenant_can_start_ingest(
    runtime: &KnowledgebaseRuntime,
) -> ApiResult<()> {
    let limits = KnowledgebaseTenantQuotaLimits::from_env();
    let inflight = runtime
        .ingestion_job_store()
        .count_inflight_jobs()
        .await
        .map_err(|error| ApiError::internal("knowledgebase_store_failed", error.to_string()))?;
    ensure_ingest_concurrency(inflight, &limits).map_err(map_tenant_quota_error)
}

pub(crate) async fn verify_ingest_capacity_after_enqueue(
    runtime: &KnowledgebaseRuntime,
) -> ApiResult<()> {
    let limits = KnowledgebaseTenantQuotaLimits::from_env();
    let inflight = runtime
        .ingestion_job_store()
        .count_inflight_jobs()
        .await
        .map_err(|error| ApiError::internal("knowledgebase_store_failed", error.to_string()))?;
    if inflight > limits.max_concurrent_ingest_jobs {
        return Err(map_tenant_quota_error(TenantQuotaExceeded {
            kind: TenantQuotaKind::IngestConcurrency,
            usage: u64::from(inflight),
            limit: u64::from(limits.max_concurrent_ingest_jobs),
        }));
    }
    Ok(())
}

pub(crate) async fn ensure_tenant_retrieval_rate(
    runtime: &KnowledgebaseRuntime,
    tenant_id: u64,
) -> ApiResult<()> {
    let limits = KnowledgebaseTenantQuotaLimits::from_env();
    let store = knowledgebase_rate_limit_store();
    let key = format!("kb:tenant:{tenant_id}:retrieval:minute");
    if store
        .check_and_record(
            &key,
            limits.max_retrievals_per_minute,
            Duration::from_secs(60),
        )
        .await
        .is_err()
    {
        return Err(map_tenant_quota_error(TenantQuotaExceeded {
            kind: TenantQuotaKind::RetrievalRate,
            usage: u64::from(limits.max_retrievals_per_minute),
            limit: u64::from(limits.max_retrievals_per_minute),
        }));
    }
    Ok(())
}
