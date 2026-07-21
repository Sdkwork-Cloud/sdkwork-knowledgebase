use async_trait::async_trait;
use sdkwork_drive_internal_sdk_generated_rust::{
    CreateRootScopeSubscriptionRequest, DriveResourceResolution, ResolveDriveResourceRequest,
    RootScopeSubscription, SdkworkCustomClient, SdkworkError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_drive_source::{
    EnsureKnowledgebaseRawScopeRequest, KnowledgeWikiDriveScope, KnowledgeWikiDriveSource,
    KnowledgeWikiDriveSourceError, KnowledgeWikiSourceResource, KnowledgebaseRawScope,
    ReadKnowledgeWikiSourceRequest, ResolveKnowledgeWikiSourceRequest,
    KNOWLEDGEBASE_RAW_CONSUMER_KIND, MAX_WIKI_SOURCE_READ_BYTES, ROOT_SCOPE_SUBSCRIPTION_TYPE,
};
use sdkwork_utils_rust::{is_blank, sha256_hash};

#[derive(Clone)]
pub struct KnowledgebaseDriveInternalSdkAdapter {
    client: SdkworkCustomClient,
}

impl KnowledgebaseDriveInternalSdkAdapter {
    pub fn new(client: SdkworkCustomClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl KnowledgeWikiDriveScope for KnowledgebaseDriveInternalSdkAdapter {
    async fn ensure_raw_scope(
        &self,
        request: EnsureKnowledgebaseRawScopeRequest,
    ) -> Result<KnowledgebaseRawScope, KnowledgeWikiDriveSourceError> {
        let drive_space_id = require_identifier(&request.drive_space_id, "drive_space_id")?;
        let knowledgebase_uuid =
            require_identifier(&request.knowledgebase_uuid, "knowledgebase_uuid")?;
        let raw_folder_node_id =
            require_identifier(&request.raw_folder_node_id, "raw_folder_node_id")?;
        let response = self
            .client
            .drive_internal_publishing()
            .root_scope_subscriptions_create(&CreateRootScopeSubscriptionRequest {
                space_id: drive_space_id,
                knowledge_base_id: knowledgebase_uuid,
                raw_folder_node_id,
            })
            .await
            .map_err(map_sdk_error)?;
        map_subscription(response)
    }

    async fn retrieve_raw_scope(
        &self,
        subscription_uuid: &str,
    ) -> Result<KnowledgebaseRawScope, KnowledgeWikiDriveSourceError> {
        let subscription_uuid = require_identifier(subscription_uuid, "subscription_uuid")?;
        let response = self
            .client
            .drive_internal_publishing()
            .root_scope_subscriptions_retrieve(&subscription_uuid)
            .await
            .map_err(map_sdk_error)?;
        map_subscription(response)
    }
}

#[async_trait]
impl KnowledgeWikiDriveSource for KnowledgebaseDriveInternalSdkAdapter {
    async fn resolve_source(
        &self,
        request: ResolveKnowledgeWikiSourceRequest,
    ) -> Result<KnowledgeWikiSourceResource, KnowledgeWikiDriveSourceError> {
        let subscription_uuid =
            require_identifier(&request.subscription_uuid, "subscription_uuid")?;
        let relative_path = require_relative_path(&request.relative_path)?;
        let response = self
            .client
            .drive_internal_publishing()
            .drive_resources_resolve(&ResolveDriveResourceRequest {
                scope_type: ROOT_SCOPE_SUBSCRIPTION_TYPE.to_string(),
                scope_uuid: subscription_uuid,
                relative_path,
                pinned_generation: normalize_optional(request.pinned_generation),
                pinned_node_version_id: normalize_optional(request.pinned_node_version_id),
            })
            .await
            .map_err(map_sdk_error)?;
        map_resource(response)
    }

    async fn read_pinned_source(
        &self,
        request: ReadKnowledgeWikiSourceRequest,
    ) -> Result<Vec<u8>, KnowledgeWikiDriveSourceError> {
        validate_read_request(&request)?;
        if request.resource.content_length == 0 {
            return Ok(Vec::new());
        }

        let range = format!("bytes=0-{}", request.resource.content_length - 1);
        let etag =
            (!request.resource.etag.trim().is_empty()).then_some(request.resource.etag.as_str());
        let bytes = self
            .client
            .drive_internal_publishing()
            .drive_resource_content_retrieve(
                &request.resource.drive_node_version_id,
                ROOT_SCOPE_SUBSCRIPTION_TYPE,
                &request.resource.subscription_uuid,
                &request.resource.normalized_relative_path,
                Some(&request.resource.scope_generation),
                Some(&range),
                etag,
                None,
                None,
                None,
                None,
            )
            .await
            .map_err(map_sdk_error)?;

        let actual_length = bytes.len() as u64;
        if actual_length != request.resource.content_length {
            return Err(KnowledgeWikiDriveSourceError::IntegrityFailed(format!(
                "pinned Drive content length {actual_length} does not match resolved length {}",
                request.resource.content_length
            )));
        }
        let actual_checksum = sha256_hash(&bytes);
        if !actual_checksum.eq_ignore_ascii_case(&request.resource.checksum_sha256_hex) {
            return Err(KnowledgeWikiDriveSourceError::IntegrityFailed(
                "pinned Drive content checksum does not match resource resolution".to_string(),
            ));
        }
        Ok(bytes)
    }
}

fn map_subscription(
    subscription: RootScopeSubscription,
) -> Result<KnowledgebaseRawScope, KnowledgeWikiDriveSourceError> {
    if subscription.consumer_kind != KNOWLEDGEBASE_RAW_CONSUMER_KIND {
        return Err(KnowledgeWikiDriveSourceError::IntegrityFailed(format!(
            "Drive root scope consumer kind must be {KNOWLEDGEBASE_RAW_CONSUMER_KIND}"
        )));
    }
    Ok(KnowledgebaseRawScope {
        subscription_uuid: require_identifier(&subscription.uuid, "subscription_uuid")?,
        drive_space_id: require_identifier(&subscription.space_id, "drive_space_id")?,
        consumer_kind: subscription.consumer_kind,
        knowledgebase_uuid: require_identifier(
            &subscription.consumer_resource_id,
            "knowledgebase_uuid",
        )?,
        raw_folder_node_id: require_identifier(&subscription.root_node_id, "raw_folder_node_id")?,
        scope_status: require_non_blank(subscription.scope_status, "scope_status")?,
        version: require_non_blank(subscription.version, "scope_version")?,
        created_at: require_non_blank(subscription.created_at, "created_at")?,
        updated_at: require_non_blank(subscription.updated_at, "updated_at")?,
    })
}

fn map_resource(
    resource: DriveResourceResolution,
) -> Result<KnowledgeWikiSourceResource, KnowledgeWikiDriveSourceError> {
    if resource.scope_type != ROOT_SCOPE_SUBSCRIPTION_TYPE {
        return Err(KnowledgeWikiDriveSourceError::IntegrityFailed(format!(
            "Drive resource scope type must be {ROOT_SCOPE_SUBSCRIPTION_TYPE}"
        )));
    }
    let content_length = resource.content_length.parse::<u64>().map_err(|_| {
        KnowledgeWikiDriveSourceError::IntegrityFailed(
            "Drive resource content_length must be an unsigned integer".to_string(),
        )
    })?;
    let checksum_sha256_hex = normalize_checksum(&resource.checksum_sha256_hex)?;
    Ok(KnowledgeWikiSourceResource {
        scope_type: resource.scope_type,
        subscription_uuid: require_identifier(&resource.scope_uuid, "subscription_uuid")?,
        scope_generation: require_non_blank(resource.scope_generation, "scope_generation")?,
        normalized_relative_path: require_relative_path(&resource.normalized_relative_path)?,
        resource_type: require_non_blank(resource.resource_type, "resource_type")?,
        drive_node_id: require_identifier(&resource.node_id, "drive_node_id")?,
        drive_node_version_id: require_identifier(
            &resource.logical_node_version_id,
            "drive_node_version_id",
        )?,
        version_no: require_non_blank(resource.version_no, "version_no")?,
        checksum_sha256_hex,
        etag: resource.etag.trim().to_string(),
        content_type: require_non_blank(resource.content_type, "content_type")?,
        content_length,
        last_modified: require_non_blank(resource.last_modified, "last_modified")?,
        scope_status: require_non_blank(resource.scope_status, "scope_status")?,
        node_status: require_non_blank(resource.node_status, "node_status")?,
        eligibility: require_non_blank(resource.eligibility, "eligibility")?,
    })
}

fn validate_read_request(
    request: &ReadKnowledgeWikiSourceRequest,
) -> Result<(), KnowledgeWikiDriveSourceError> {
    if request.maximum_bytes == 0 || request.maximum_bytes > MAX_WIKI_SOURCE_READ_BYTES {
        return Err(KnowledgeWikiDriveSourceError::InvalidRequest(format!(
            "maximum_bytes must be between 1 and {MAX_WIKI_SOURCE_READ_BYTES}"
        )));
    }
    if request.resource.content_length > request.maximum_bytes {
        return Err(KnowledgeWikiDriveSourceError::InvalidRequest(format!(
            "Drive resource size {} exceeds read limit {}",
            request.resource.content_length, request.maximum_bytes
        )));
    }
    if request.resource.scope_type != ROOT_SCOPE_SUBSCRIPTION_TYPE {
        return Err(KnowledgeWikiDriveSourceError::InvalidRequest(
            "only ROOT_SCOPE_SUBSCRIPTION resources may be read".to_string(),
        ));
    }
    require_identifier(&request.resource.subscription_uuid, "subscription_uuid")?;
    require_identifier(
        &request.resource.drive_node_version_id,
        "drive_node_version_id",
    )?;
    require_non_blank(
        request.resource.scope_generation.clone(),
        "scope_generation",
    )?;
    require_relative_path(&request.resource.normalized_relative_path)?;
    normalize_checksum(&request.resource.checksum_sha256_hex)?;
    Ok(())
}

fn require_identifier(
    value: &str,
    field_name: &str,
) -> Result<String, KnowledgeWikiDriveSourceError> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 160
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':'))
    {
        return Err(KnowledgeWikiDriveSourceError::InvalidRequest(format!(
            "invalid {field_name}"
        )));
    }
    Ok(value.to_string())
}

fn require_relative_path(value: &str) -> Result<String, KnowledgeWikiDriveSourceError> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 2048
        || value.starts_with('/')
        || value.starts_with('\\')
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || value
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(KnowledgeWikiDriveSourceError::InvalidRequest(
            "relative_path must be a normalized path below sources/raw".to_string(),
        ));
    }
    Ok(value.to_string())
}

fn normalize_checksum(value: &str) -> Result<String, KnowledgeWikiDriveSourceError> {
    let value = value.trim().to_ascii_lowercase();
    let value = value.strip_prefix("sha256:").unwrap_or(&value);
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(KnowledgeWikiDriveSourceError::IntegrityFailed(
            "Drive resource checksum must be a SHA-256 hex digest".to_string(),
        ));
    }
    Ok(value.to_string())
}

fn require_non_blank(
    value: String,
    field_name: &str,
) -> Result<String, KnowledgeWikiDriveSourceError> {
    if is_blank(Some(&value)) {
        return Err(KnowledgeWikiDriveSourceError::IntegrityFailed(format!(
            "Drive response {field_name} is required"
        )));
    }
    Ok(value.trim().to_string())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn map_sdk_error(error: SdkworkError) -> KnowledgeWikiDriveSourceError {
    match error {
        SdkworkError::HttpStatus {
            status: 400 | 422, ..
        } => KnowledgeWikiDriveSourceError::InvalidRequest(
            "Drive Internal API rejected the request".to_string(),
        ),
        SdkworkError::HttpStatus { status: 404, .. } => KnowledgeWikiDriveSourceError::NotFound(
            "Drive root scope or resource was not found".to_string(),
        ),
        SdkworkError::HttpStatus { status: 409, .. } => KnowledgeWikiDriveSourceError::Conflict(
            "Drive root scope or resource changed concurrently".to_string(),
        ),
        SdkworkError::HttpStatus { status, .. } => KnowledgeWikiDriveSourceError::Upstream(
            format!("Drive Internal API returned HTTP status {status}"),
        ),
        SdkworkError::ApiStatus { code, trace_id } => KnowledgeWikiDriveSourceError::Upstream(
            format!("Drive Internal API returned code {code} (traceId={trace_id})"),
        ),
        SdkworkError::ResponseBodyTooLarge { maximum_bytes } => {
            KnowledgeWikiDriveSourceError::Upstream(format!(
                "Drive Internal API response exceeds {maximum_bytes} bytes"
            ))
        }
        _ => KnowledgeWikiDriveSourceError::Upstream(
            "Drive Internal API transport or response validation failed".to_string(),
        ),
    }
}
