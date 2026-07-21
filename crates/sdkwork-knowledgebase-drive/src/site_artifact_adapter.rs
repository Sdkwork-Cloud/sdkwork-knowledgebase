use std::sync::Arc;

use async_trait::async_trait;
use sdkwork_drive_storage_contract::{
    DriveByteRange, DriveObjectLocator, DriveObjectStore, ReadObjectRangeRequest,
};
use sdkwork_drive_uploader_service::service::{
    DriveUploaderService, PrepareUploaderUploadCommand, SqlUploaderStore, UploadBytesCommand,
    UploaderActor, UploaderRetention, UploaderTarget,
};
use sdkwork_drive_workspace_service::application::workspace_service::{
    GetDriveWorkspaceNodeCommand, SqlDriveWorkspaceService,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_site_artifact_store::{
    KnowledgeSiteArtifact, KnowledgeSiteArtifactRef, KnowledgeSiteArtifactStore,
    KnowledgeSiteArtifactStoreError, ReadKnowledgeSiteArtifactRequest,
    WriteKnowledgeSiteArtifactRequest,
};
use sdkwork_utils_rust::{is_blank, sha256_hash};
use sqlx::AnyPool;
use time::OffsetDateTime;

const APP_ID: &str = "sdkwork-knowledgebase";
const MAX_ARTIFACT_WRITE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug)]
pub struct KnowledgebaseDriveSiteArtifactStore<S>
where
    S: DriveObjectStore,
{
    drive_pool: AnyPool,
    object_store: Arc<S>,
}

impl<S> KnowledgebaseDriveSiteArtifactStore<S>
where
    S: DriveObjectStore,
{
    pub fn new(drive_pool: AnyPool, object_store: Arc<S>) -> Self {
        Self {
            drive_pool,
            object_store,
        }
    }
}

#[async_trait]
impl<S> KnowledgeSiteArtifactStore for KnowledgebaseDriveSiteArtifactStore<S>
where
    S: DriveObjectStore + 'static,
{
    async fn write_artifact(
        &self,
        request: WriteKnowledgeSiteArtifactRequest,
    ) -> Result<KnowledgeSiteArtifactRef, KnowledgeSiteArtifactStoreError> {
        validate_write(&request)?;
        let checksum = sha256_hash(&request.body);
        let stable_key = sha256_hash(
            format!(
                "{}:{}:{}:{}",
                request.tenant_id, request.site_id, request.release_id, request.public_path
            )
            .as_bytes(),
        );
        let upload_id = format!("kb-site-{}", &stable_key[..32]);
        let now_epoch_ms = now_epoch_ms()?;
        let content_length = i64::try_from(request.body.len()).map_err(|_| {
            KnowledgeSiteArtifactStoreError::InvalidRequest(
                "site artifact content length exceeds signed int64 range".to_string(),
            )
        })?;
        let uploader = DriveUploaderService::new(SqlUploaderStore::new(self.drive_pool.clone()));
        let item = uploader
            .upload_bytes(
                self.object_store.as_ref(),
                UploadBytesCommand {
                    prepare: PrepareUploaderUploadCommand {
                        id: upload_id.clone(),
                        task_id: upload_id,
                        tenant_id: request.tenant_id.to_string(),
                        organization_id: Some(request.organization_id.to_string()),
                        actor: UploaderActor::System {
                            operator_id: request.operator_id.clone(),
                        },
                        app_id: APP_ID.to_string(),
                        app_resource_type: "knowledge_site_release".to_string(),
                        app_resource_id: request.release_id.to_string(),
                        scene: Some("knowledge_site_publication".to_string()),
                        source: Some("server_generated".to_string()),
                        upload_profile_code: profile_for(&request.content_type).to_string(),
                        file_fingerprint: checksum.clone(),
                        original_file_name: request.file_name,
                        content_type: request.content_type.clone(),
                        content_length,
                        chunk_size_bytes: content_length.max(1),
                        target: UploaderTarget::AiGeneratedSpace {
                            parent_node_id: None,
                        },
                        retention: UploaderRetention::LongTerm,
                        operator_id: request.operator_id,
                        now_epoch_ms,
                    },
                    body: request.body,
                    uploaded_at_epoch_ms: now_epoch_ms,
                },
            )
            .await
            .map_err(map_drive_service_error)?;

        let stored_checksum = item
            .checksum_sha256_hex
            .as_deref()
            .and_then(normalize_sha256_hex);
        if stored_checksum.as_deref() != Some(checksum.as_str()) {
            return Err(KnowledgeSiteArtifactStoreError::IntegrityFailed(
                "Drive uploader checksum does not match generated artifact".to_string(),
            ));
        }
        Ok(KnowledgeSiteArtifactRef {
            drive_uri: stable_drive_uri(&item.space_id, &item.node_id),
            drive_space_id: item.space_id,
            drive_node_id: item.node_id,
            content_type: item.content_type,
            content_length: u64::try_from(item.content_length).map_err(|_| {
                KnowledgeSiteArtifactStoreError::Internal(
                    "Drive uploader returned a negative content length".to_string(),
                )
            })?,
            checksum_sha256_hex: checksum,
        })
    }

    async fn read_artifact(
        &self,
        request: ReadKnowledgeSiteArtifactRequest,
    ) -> Result<KnowledgeSiteArtifact, KnowledgeSiteArtifactStoreError> {
        if request.tenant_id == 0
            || is_blank(Some(request.drive_space_id.as_str()))
            || is_blank(Some(request.drive_node_id.as_str()))
            || request.max_bytes == 0
        {
            return Err(KnowledgeSiteArtifactStoreError::InvalidRequest(
                "tenant, Drive space/node, and max_bytes are required".to_string(),
            ));
        }
        let workspace = SqlDriveWorkspaceService::new(self.drive_pool.clone());
        let node = workspace
            .get_node(GetDriveWorkspaceNodeCommand {
                tenant_id: request.tenant_id.to_string(),
                space_id: request.drive_space_id.clone(),
                node_id: request.drive_node_id.clone(),
            })
            .await
            .map_err(map_drive_service_error)?;
        if node.is_none() {
            return Err(KnowledgeSiteArtifactStoreError::NotFound);
        }
        let object = workspace
            .find_latest_active_storage_object_by_node(
                &request.tenant_id.to_string(),
                &request.drive_node_id,
            )
            .await
            .map_err(map_drive_service_error)?
            .ok_or(KnowledgeSiteArtifactStoreError::NotFound)?;
        if object.content_length <= 0
            || u64::try_from(object.content_length).unwrap_or(u64::MAX) > request.max_bytes
        {
            return Err(KnowledgeSiteArtifactStoreError::InvalidRequest(
                "site artifact exceeds the configured read bound".to_string(),
            ));
        }
        let expected_length = u64::try_from(object.content_length).map_err(|_| {
            KnowledgeSiteArtifactStoreError::Internal(
                "Drive storage object has a negative content length".to_string(),
            )
        })?;
        let (_, mut stream) = self
            .object_store
            .read_object_range(ReadObjectRangeRequest {
                locator: DriveObjectLocator {
                    bucket: object.bucket,
                    object_key: object.object_key,
                },
                range: DriveByteRange {
                    start_inclusive: 0,
                    end_inclusive: expected_length - 1,
                },
            })
            .await
            .map_err(map_object_store_error)?;
        let mut body = Vec::with_capacity(expected_length as usize);
        while let Some(chunk) = stream.next_chunk().await.map_err(map_object_store_error)? {
            if body.len().saturating_add(chunk.len()) > request.max_bytes as usize {
                return Err(KnowledgeSiteArtifactStoreError::InvalidRequest(
                    "site artifact stream exceeds the configured read bound".to_string(),
                ));
            }
            body.extend_from_slice(&chunk);
        }
        let stored_checksum = normalize_sha256_hex(&object.checksum_sha256_hex).ok_or_else(|| {
            KnowledgeSiteArtifactStoreError::IntegrityFailed(
                "Drive site artifact checksum is not a valid SHA-256 digest".to_string(),
            )
        })?;
        if body.len() as u64 != expected_length || sha256_hash(&body) != stored_checksum {
            return Err(KnowledgeSiteArtifactStoreError::IntegrityFailed(
                "Drive site artifact length or checksum mismatch".to_string(),
            ));
        }
        Ok(KnowledgeSiteArtifact {
            content_type: object.content_type,
            checksum_sha256_hex: stored_checksum,
            body,
        })
    }
}

fn validate_write(
    request: &WriteKnowledgeSiteArtifactRequest,
) -> Result<(), KnowledgeSiteArtifactStoreError> {
    if request.tenant_id == 0
        || request.organization_id == 0
        || request.site_id == 0
        || request.release_id == 0
        || is_blank(Some(request.operator_id.as_str()))
        || is_blank(Some(request.public_path.as_str()))
        || is_blank(Some(request.file_name.as_str()))
        || is_blank(Some(request.content_type.as_str()))
    {
        return Err(KnowledgeSiteArtifactStoreError::InvalidRequest(
            "site artifact ownership, path, filename, and content type are required".to_string(),
        ));
    }
    if request.body.is_empty() || request.body.len() > MAX_ARTIFACT_WRITE_BYTES {
        return Err(KnowledgeSiteArtifactStoreError::InvalidRequest(format!(
            "site artifact must contain 1 through {MAX_ARTIFACT_WRITE_BYTES} bytes"
        )));
    }
    if request.file_name.contains('/')
        || request.file_name.contains('\\')
        || request.file_name == "."
        || request.file_name == ".."
    {
        return Err(KnowledgeSiteArtifactStoreError::InvalidRequest(
            "site artifact file_name must be a basename".to_string(),
        ));
    }
    Ok(())
}

fn profile_for(content_type: &str) -> &'static str {
    if content_type.starts_with("text/") || content_type.contains("json") {
        "text"
    } else if content_type.starts_with("image/") {
        "image"
    } else {
        "generic"
    }
}

fn stable_drive_uri(space_id: &str, node_id: &str) -> String {
    format!("drive://spaces/{space_id}/nodes/{node_id}")
}

fn normalize_sha256_hex(value: &str) -> Option<String> {
    let value = value
        .get(..7)
        .filter(|prefix| prefix.eq_ignore_ascii_case("sha256:"))
        .map(|_| &value[7..])
        .unwrap_or(value);
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    Some(value.to_ascii_lowercase())
}

fn now_epoch_ms() -> Result<i64, KnowledgeSiteArtifactStoreError> {
    i64::try_from(OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000).map_err(|_| {
        KnowledgeSiteArtifactStoreError::Internal(
            "current time exceeds signed epoch millisecond range".to_string(),
        )
    })
}

fn map_drive_service_error(
    error: sdkwork_drive_workspace_service::DriveServiceError,
) -> KnowledgeSiteArtifactStoreError {
    match error {
        sdkwork_drive_workspace_service::DriveServiceError::Validation(detail) => {
            KnowledgeSiteArtifactStoreError::InvalidRequest(detail)
        }
        sdkwork_drive_workspace_service::DriveServiceError::NotFound(_) => {
            KnowledgeSiteArtifactStoreError::NotFound
        }
        sdkwork_drive_workspace_service::DriveServiceError::Conflict(detail)
        | sdkwork_drive_workspace_service::DriveServiceError::PermissionDenied(detail)
        | sdkwork_drive_workspace_service::DriveServiceError::Internal(detail) => {
            KnowledgeSiteArtifactStoreError::Internal(detail)
        }
    }
}

fn map_object_store_error(
    error: sdkwork_drive_storage_contract::DriveObjectStoreError,
) -> KnowledgeSiteArtifactStoreError {
    use sdkwork_drive_storage_contract::DriveObjectStoreErrorKind;
    match error.kind {
        DriveObjectStoreErrorKind::NotFound => KnowledgeSiteArtifactStoreError::NotFound,
        DriveObjectStoreErrorKind::InvalidRequest => {
            KnowledgeSiteArtifactStoreError::InvalidRequest(error.message)
        }
        DriveObjectStoreErrorKind::IntegrityFailed => {
            KnowledgeSiteArtifactStoreError::IntegrityFailed(error.message)
        }
        _ => KnowledgeSiteArtifactStoreError::Internal(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_sha256_hex;

    const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn normalizes_supported_sha256_representations() {
        assert_eq!(normalize_sha256_hex(DIGEST).as_deref(), Some(DIGEST));
        assert_eq!(
            normalize_sha256_hex(&format!("sha256:{DIGEST}")).as_deref(),
            Some(DIGEST)
        );
        assert_eq!(
            normalize_sha256_hex(&format!("SHA256:{}", DIGEST.to_ascii_uppercase())).as_deref(),
            Some(DIGEST)
        );
    }

    #[test]
    fn rejects_malformed_sha256_representations() {
        assert_eq!(normalize_sha256_hex(""), None);
        assert_eq!(normalize_sha256_hex("sha256:abc"), None);
        assert_eq!(
            normalize_sha256_hex(
                "g123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            ),
            None
        );
        assert_eq!(normalize_sha256_hex(&format!(" {DIGEST}")), None);
    }
}
