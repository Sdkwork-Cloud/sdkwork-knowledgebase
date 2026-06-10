use async_trait::async_trait;
use sdkwork_drive_product::application::space_service::{
    CreateSpaceCommand, DeleteSpaceCommand, GetSpaceCommand, ListSpacesCommand,
    SqlDriveSpaceService,
};
use sdkwork_drive_product::application::workspace_service::{
    DriveWorkspaceChildrenPage, DriveWorkspaceNode, DriveWorkspaceNodeKind,
    DriveWorkspaceObjectRef, EnsureDriveWorkspaceNode, EnsureDriveWorkspaceNodesCommand,
    GetDriveWorkspaceNodeCommand, ListDriveWorkspaceChildrenCommand,
    ResolveDriveWorkspacePathCommand, SqlDriveWorkspaceService,
};
use sdkwork_drive_product::domain::space::DriveSpaceType;
use sdkwork_drive_product::DriveProductError;
use sdkwork_drive_storage_contract::{
    DriveByteRange, DriveObjectLocator, DriveObjectStore, DriveObjectStoreError,
    DriveObjectStoreErrorKind, HeadObjectRequest, PutObjectRequest, ReadObjectRangeRequest,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_node_tree::{
    DriveNodeKind, GetKnowledgeDriveNodeRequest, KnowledgeDriveNodePage, KnowledgeDriveNodeSummary,
    KnowledgeDriveNodeTree, KnowledgeDriveNodeTreeError, ListKnowledgeDriveNodeChildrenRequest,
    ResolveKnowledgeDriveNodePathRequest,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_space::{
    CreateKnowledgeDriveSpaceRequest, DeleteKnowledgeDriveSpaceRequest, KnowledgeDriveSpaceBinding,
    KnowledgeDriveSpaceProvisioner, KnowledgeDriveSpaceProvisionerError,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_workspace::{
    EnsureKnowledgeDriveNodeKind, EnsureKnowledgeDriveNodeRequest,
    EnsureKnowledgeDriveNodesRequest, KnowledgeDriveWorkspace, KnowledgeDriveWorkspaceError,
};
use sha2::{Digest, Sha256};
use sqlx::AnyPool;
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct KnowledgebaseDriveStorageAdapter {
    store: Arc<dyn DriveObjectStore>,
    storage_provider_id: String,
    bucket: String,
    object_key_root: String,
}

#[derive(Debug, Clone)]
pub struct KnowledgebaseDriveSpaceProvisionerAdapter {
    pool: AnyPool,
}

impl KnowledgebaseDriveSpaceProvisionerAdapter {
    pub fn new(pool: AnyPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Clone)]
pub struct KnowledgebaseDriveWorkspaceAdapter {
    pool: AnyPool,
    tenant_id: String,
    operator_id: String,
}

impl KnowledgebaseDriveWorkspaceAdapter {
    pub fn new(
        pool: AnyPool,
        tenant_id: impl Into<String>,
        operator_id: impl Into<String>,
    ) -> Self {
        Self {
            pool,
            tenant_id: tenant_id.into(),
            operator_id: operator_id.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KnowledgebaseDriveNodeTreeAdapter {
    pool: AnyPool,
    tenant_id: String,
}

impl KnowledgebaseDriveNodeTreeAdapter {
    pub fn new(pool: AnyPool, tenant_id: impl Into<String>) -> Self {
        Self {
            pool,
            tenant_id: tenant_id.into(),
        }
    }
}

impl KnowledgebaseDriveStorageAdapter {
    pub fn new<S>(
        store: Arc<S>,
        storage_provider_id: impl Into<String>,
        bucket: impl Into<String>,
        object_key_root: impl Into<String>,
    ) -> Self
    where
        S: DriveObjectStore + 'static,
    {
        Self {
            store,
            storage_provider_id: storage_provider_id.into(),
            bucket: bucket.into(),
            object_key_root: trim_slashes(&object_key_root.into()),
        }
    }

    fn locator_for(&self, logical_path: &str) -> Result<DriveObjectLocator, KnowledgeStorageError> {
        let safe_logical_path = safe_logical_path(logical_path)?;
        let object_key = if self.object_key_root.is_empty() {
            safe_logical_path
        } else {
            format!("{}/{}", self.object_key_root, safe_logical_path)
        };

        Ok(DriveObjectLocator {
            bucket: self.bucket.clone(),
            object_key,
        })
    }
}

#[async_trait]
impl KnowledgeDriveSpaceProvisioner for KnowledgebaseDriveSpaceProvisionerAdapter {
    async fn create_knowledge_drive_space(
        &self,
        request: CreateKnowledgeDriveSpaceRequest,
    ) -> Result<KnowledgeDriveSpaceBinding, KnowledgeDriveSpaceProvisionerError> {
        let tenant_id = safe_drive_identifier(&request.tenant_id, "tenant_id")
            .map_err(KnowledgeDriveSpaceProvisionerError::InvalidRequest)?;
        let owner_subject_type = require_drive_owner_subject_type(&request.owner_subject_type)?;
        let owner_subject_id = safe_drive_owner_subject_id(&request.owner_subject_id)?;
        let display_name = require_non_empty(request.display_name, "display_name")?;
        let operator_id = safe_drive_identifier(&request.operator_id, "operator_id")
            .map_err(KnowledgeDriveSpaceProvisionerError::InvalidRequest)?;

        let service = self.space_service();
        if let Some(existing) = find_existing_knowledge_space(
            &service,
            &tenant_id,
            &owner_subject_type,
            &owner_subject_id,
        )
        .await?
        {
            return Ok(KnowledgeDriveSpaceBinding {
                drive_space_id: existing,
            });
        }

        let drive_space_id = drive_space_id_for_knowledge_space(&request.knowledge_space_uuid)?;
        match service
            .create_space(CreateSpaceCommand {
                id: drive_space_id.clone(),
                tenant_id: tenant_id.clone(),
                owner_subject_type: owner_subject_type.clone(),
                owner_subject_id: owner_subject_id.clone(),
                display_name,
                space_type: DriveSpaceType::KnowledgeBase,
                operator_id,
            })
            .await
        {
            Ok(space) => Ok(KnowledgeDriveSpaceBinding {
                drive_space_id: space.id,
            }),
            Err(DriveProductError::Conflict(_)) => {
                let Some(existing) = find_existing_knowledge_space(
                    &service,
                    &tenant_id,
                    &owner_subject_type,
                    &owner_subject_id,
                )
                .await?
                else {
                    return Err(KnowledgeDriveSpaceProvisionerError::Upstream(
                        "drive knowledge space conflict could not be resolved".to_string(),
                    ));
                };
                Ok(KnowledgeDriveSpaceBinding {
                    drive_space_id: existing,
                })
            }
            Err(error) => Err(map_space_product_error(error)),
        }
    }

    async fn delete_knowledge_drive_space(
        &self,
        request: DeleteKnowledgeDriveSpaceRequest,
    ) -> Result<(), KnowledgeDriveSpaceProvisionerError> {
        let tenant_id = safe_drive_identifier(&request.tenant_id, "tenant_id")
            .map_err(KnowledgeDriveSpaceProvisionerError::InvalidRequest)?;
        let drive_space_id = safe_drive_identifier(&request.drive_space_id, "drive_space_id")
            .map_err(KnowledgeDriveSpaceProvisionerError::InvalidRequest)?;
        let owner_subject_type = require_drive_owner_subject_type(&request.owner_subject_type)?;
        let owner_subject_id = safe_drive_owner_subject_id(&request.owner_subject_id)?;
        let operator_id = safe_drive_identifier(&request.operator_id, "operator_id")
            .map_err(KnowledgeDriveSpaceProvisionerError::InvalidRequest)?;

        let service = self.space_service();
        let drive_space = match service
            .get_space(GetSpaceCommand {
                tenant_id: tenant_id.clone(),
                space_id: drive_space_id.clone(),
            })
            .await
        {
            Ok(space) => space,
            Err(DriveProductError::NotFound(_)) => return Ok(()),
            Err(error) => return Err(map_space_product_error(error)),
        };

        if drive_space.space_type != DriveSpaceType::KnowledgeBase
            || drive_space.owner_subject_type != owner_subject_type
            || drive_space.owner_subject_id != owner_subject_id
        {
            return Err(KnowledgeDriveSpaceProvisionerError::InvalidRequest(
                "drive space does not belong to the requested knowledge space owner".to_string(),
            ));
        }

        match service
            .delete_space(DeleteSpaceCommand {
                tenant_id,
                space_id: drive_space_id,
                operator_id,
            })
            .await
        {
            Ok(_) | Err(DriveProductError::NotFound(_)) => Ok(()),
            Err(error) => Err(map_space_product_error(error)),
        }
    }
}

impl KnowledgebaseDriveSpaceProvisionerAdapter {
    fn space_service(&self) -> SqlDriveSpaceService {
        SqlDriveSpaceService::new(self.pool.clone())
    }
}

#[async_trait]
impl KnowledgeDriveStorage for KnowledgebaseDriveStorageAdapter {
    async fn put_object(
        &self,
        request: PutKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        let locator = self.locator_for(&request.logical_path)?;
        let size_bytes = request.body.len() as u64;
        let computed_checksum_sha256_hex = checksum_sha256_hex(&request.body);
        let checksum_sha256_hex = verified_request_checksum(
            request.checksum_sha256_hex.as_deref(),
            &computed_checksum_sha256_hex,
        )?;
        let mut metadata = BTreeMap::new();
        metadata.insert("logical_path".to_string(), request.logical_path.clone());
        metadata.insert("object_role".to_string(), request.object_role.clone());

        let response = self
            .store
            .put_object(PutObjectRequest {
                locator: locator.clone(),
                content_type: Some(request.content_type.clone()),
                metadata,
                body: request.body,
                checksum_sha256_hex: Some(checksum_sha256_hex.clone()),
            })
            .await
            .map_err(map_drive_error)?;
        let version_id = content_version_id(response.version_id, &checksum_sha256_hex);

        Ok(KnowledgeObjectRef {
            storage_provider_id: self.storage_provider_id.clone(),
            bucket: response.locator.bucket,
            object_key: response.locator.object_key,
            logical_path: request.logical_path,
            object_role: request.object_role,
            content_type: request.content_type,
            size_bytes,
            checksum_sha256_hex: Some(checksum_sha256_hex),
            etag: response.etag,
            version_id: Some(version_id),
        })
    }

    async fn head_object(
        &self,
        request: HeadKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        if let Some(storage_provider_id) = request.storage_provider_id.as_deref() {
            if storage_provider_id != self.storage_provider_id {
                return Err(KnowledgeStorageError::InvalidRequest(format!(
                    "storage_provider_id does not match adapter provider: {storage_provider_id}"
                )));
            }
        }
        let logical_path = request
            .logical_path
            .clone()
            .unwrap_or_else(|| request.object_key.clone());
        let locator = if request.bucket.is_empty() {
            self.locator_for(&logical_path)?
        } else {
            DriveObjectLocator {
                bucket: request.bucket,
                object_key: request.object_key,
            }
        };
        let response = self
            .store
            .head_object(HeadObjectRequest { locator })
            .await
            .map_err(map_drive_error)?;
        let version_id = content_version_id_from_head(
            response.version_id,
            response.checksum_sha256_hex.as_deref(),
        );

        Ok(KnowledgeObjectRef {
            storage_provider_id: self.storage_provider_id.clone(),
            bucket: response.locator.bucket,
            object_key: response.locator.object_key,
            logical_path,
            object_role: request.object_role,
            content_type: response
                .content_type
                .unwrap_or_else(|| "application/octet-stream".to_string()),
            size_bytes: response.content_length,
            checksum_sha256_hex: response.checksum_sha256_hex,
            etag: response.etag,
            version_id,
        })
    }

    async fn get_object_text(
        &self,
        object_ref: &KnowledgeObjectRef,
    ) -> Result<String, KnowledgeStorageError> {
        if object_ref.size_bytes == 0 {
            return Ok(String::new());
        }

        let end_inclusive = object_ref.size_bytes.saturating_sub(1);
        let (_, mut stream) = self
            .store
            .read_object_range(ReadObjectRangeRequest {
                locator: DriveObjectLocator {
                    bucket: object_ref.bucket.clone(),
                    object_key: object_ref.object_key.clone(),
                },
                range: DriveByteRange {
                    start_inclusive: 0,
                    end_inclusive,
                },
            })
            .await
            .map_err(map_drive_error)?;

        let mut bytes = Vec::new();
        while let Some(chunk) = stream.next_chunk().await.map_err(map_drive_error)? {
            bytes.extend_from_slice(&chunk);
        }

        String::from_utf8(bytes)
            .map_err(|error| KnowledgeStorageError::InvalidRequest(error.to_string()))
    }
}

#[async_trait]
impl KnowledgeDriveWorkspace for KnowledgebaseDriveWorkspaceAdapter {
    async fn ensure_nodes(
        &self,
        request: EnsureKnowledgeDriveNodesRequest,
    ) -> Result<(), KnowledgeDriveWorkspaceError> {
        let drive_space_id = safe_drive_id(&request.drive_space_id, "drive_space_id")?;
        if request.nodes.is_empty() {
            return Ok(());
        }

        let nodes = request
            .nodes
            .into_iter()
            .map(knowledge_node_to_drive_node)
            .collect::<Result<Vec<_>, _>>()?;
        self.workspace_service()
            .ensure_nodes(EnsureDriveWorkspaceNodesCommand {
                tenant_id: self.tenant_id.clone(),
                space_id: drive_space_id,
                operator_id: self.operator_id.clone(),
                nodes,
            })
            .await
            .map_err(map_workspace_product_error)
    }
}

impl KnowledgebaseDriveWorkspaceAdapter {
    fn workspace_service(&self) -> SqlDriveWorkspaceService {
        SqlDriveWorkspaceService::new(self.pool.clone())
    }
}

#[async_trait]
impl KnowledgeDriveNodeTree for KnowledgebaseDriveNodeTreeAdapter {
    async fn resolve_path(
        &self,
        request: ResolveKnowledgeDriveNodePathRequest,
    ) -> Result<Option<KnowledgeDriveNodeSummary>, KnowledgeDriveNodeTreeError> {
        let drive_space_id = safe_drive_id(&request.drive_space_id, "drive_space_id")
            .map_err(|error| KnowledgeDriveNodeTreeError::InvalidRequest(error.to_string()))?;
        let logical_path = safe_logical_path(&request.logical_path)
            .map_err(|error| KnowledgeDriveNodeTreeError::InvalidRequest(error.to_string()))?;
        self.workspace_service()
            .resolve_path(ResolveDriveWorkspacePathCommand {
                tenant_id: self.tenant_id.clone(),
                space_id: drive_space_id,
                logical_path,
            })
            .await
            .map_err(map_tree_product_error)?
            .map(knowledge_summary_from_drive_node)
            .transpose()
    }

    async fn get_node(
        &self,
        request: GetKnowledgeDriveNodeRequest,
    ) -> Result<Option<KnowledgeDriveNodeSummary>, KnowledgeDriveNodeTreeError> {
        let drive_space_id = safe_drive_id(&request.drive_space_id, "drive_space_id")
            .map_err(|error| KnowledgeDriveNodeTreeError::InvalidRequest(error.to_string()))?;
        let drive_node_id = safe_drive_id(&request.drive_node_id, "drive_node_id")
            .map_err(|error| KnowledgeDriveNodeTreeError::InvalidRequest(error.to_string()))?;
        self.workspace_service()
            .get_node(GetDriveWorkspaceNodeCommand {
                tenant_id: self.tenant_id.clone(),
                space_id: drive_space_id,
                node_id: drive_node_id,
            })
            .await
            .map_err(map_tree_product_error)?
            .map(knowledge_summary_from_drive_node)
            .transpose()
    }

    async fn list_children(
        &self,
        request: ListKnowledgeDriveNodeChildrenRequest,
    ) -> Result<KnowledgeDriveNodePage, KnowledgeDriveNodeTreeError> {
        let drive_space_id = safe_drive_id(&request.drive_space_id, "drive_space_id")
            .map_err(|error| KnowledgeDriveNodeTreeError::InvalidRequest(error.to_string()))?;
        let page_size = request.page_size.clamp(1, 200);
        let offset = decode_cursor(request.cursor.as_deref())?;

        let page = self
            .workspace_service()
            .list_children(ListDriveWorkspaceChildrenCommand {
                tenant_id: self.tenant_id.clone(),
                space_id: drive_space_id,
                parent_node_id: request.parent_drive_node_id,
                offset,
                page_size: i64::from(page_size),
            })
            .await
            .map_err(map_tree_product_error)?;
        knowledge_page_from_drive_page(page)
    }
}

impl KnowledgebaseDriveNodeTreeAdapter {
    fn workspace_service(&self) -> SqlDriveWorkspaceService {
        SqlDriveWorkspaceService::new(self.pool.clone())
    }
}

fn trim_slashes(value: &str) -> String {
    value.trim_matches('/').replace('\\', "/")
}

fn safe_logical_path(value: &str) -> Result<String, KnowledgeStorageError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.starts_with('/')
        || trimmed.starts_with('\\')
        || trimmed.contains(':')
    {
        return Err(KnowledgeStorageError::InvalidRequest(format!(
            "unsafe logical_path: {value}"
        )));
    }

    let normalized = trimmed.replace('\\', "/");
    let mut segments = Vec::new();
    for segment in normalized.split('/') {
        if segment.is_empty()
            || segment == "."
            || segment == ".."
            || !segment
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.')
        {
            return Err(KnowledgeStorageError::InvalidRequest(format!(
                "unsafe logical_path: {value}"
            )));
        }
        segments.push(segment);
    }

    Ok(segments.join("/"))
}

fn map_drive_error(error: DriveObjectStoreError) -> KnowledgeStorageError {
    match error.kind {
        DriveObjectStoreErrorKind::NotFound => KnowledgeStorageError::NotFound(error.message),
        DriveObjectStoreErrorKind::InvalidRequest => {
            KnowledgeStorageError::InvalidRequest(error.message)
        }
        DriveObjectStoreErrorKind::IntegrityFailed => {
            KnowledgeStorageError::IntegrityFailed(error.message)
        }
        DriveObjectStoreErrorKind::PermissionDenied
        | DriveObjectStoreErrorKind::Timeout
        | DriveObjectStoreErrorKind::Unavailable
        | DriveObjectStoreErrorKind::RateLimited
        | DriveObjectStoreErrorKind::Conflict
        | DriveObjectStoreErrorKind::UpstreamError
        | DriveObjectStoreErrorKind::NotSupported => KnowledgeStorageError::Upstream(error.message),
        DriveObjectStoreErrorKind::Internal => KnowledgeStorageError::Internal(error.message),
    }
}

fn safe_drive_id(value: &str, field_name: &str) -> Result<String, KnowledgeDriveWorkspaceError> {
    safe_drive_identifier(value, field_name).map_err(KnowledgeDriveWorkspaceError::InvalidRequest)
}

async fn find_existing_knowledge_space(
    service: &SqlDriveSpaceService,
    tenant_id: &str,
    owner_subject_type: &str,
    owner_subject_id: &str,
) -> Result<Option<String>, KnowledgeDriveSpaceProvisionerError> {
    let spaces = service
        .list_spaces(ListSpacesCommand {
            tenant_id: tenant_id.to_string(),
            owner_subject_type: Some(owner_subject_type.to_string()),
            owner_subject_id: Some(owner_subject_id.to_string()),
        })
        .await
        .map_err(map_space_product_error)?;
    Ok(spaces
        .into_iter()
        .find(|space| space.space_type == DriveSpaceType::KnowledgeBase)
        .map(|space| space.id))
}

fn require_non_empty(
    value: String,
    field_name: &str,
) -> Result<String, KnowledgeDriveSpaceProvisionerError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(KnowledgeDriveSpaceProvisionerError::InvalidRequest(
            format!("{field_name} is required"),
        ));
    }
    Ok(value)
}

fn require_drive_owner_subject_type(
    value: &str,
) -> Result<String, KnowledgeDriveSpaceProvisionerError> {
    let value = value.trim();
    match value {
        "app" | "user" | "group" | "organization" => Ok(value.to_string()),
        _ => Err(KnowledgeDriveSpaceProvisionerError::InvalidRequest(
            "owner_subject_type must be app, user, group, or organization".to_string(),
        )),
    }
}

fn safe_drive_owner_subject_id(value: &str) -> Result<String, KnowledgeDriveSpaceProvisionerError> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 128
        || !value.chars().all(|ch| {
            ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' || ch == ':'
        })
    {
        return Err(KnowledgeDriveSpaceProvisionerError::InvalidRequest(
            "invalid owner_subject_id".to_string(),
        ));
    }
    Ok(value.to_string())
}

fn drive_space_id_for_knowledge_space(
    knowledge_space_uuid: &str,
) -> Result<String, KnowledgeDriveSpaceProvisionerError> {
    let safe_uuid = safe_drive_identifier(knowledge_space_uuid, "knowledge_space_uuid")
        .map_err(KnowledgeDriveSpaceProvisionerError::InvalidRequest)?;
    let drive_space_id = format!("kb-{safe_uuid}");
    if drive_space_id.len() > 64 {
        return Err(KnowledgeDriveSpaceProvisionerError::InvalidRequest(
            "knowledge_space_uuid is too long for drive space id".to_string(),
        ));
    }
    Ok(drive_space_id)
}

fn safe_drive_identifier(value: &str, field_name: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 128
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.')
    {
        return Err(format!("invalid {field_name}"));
    }
    Ok(value.to_string())
}

fn knowledge_node_to_drive_node(
    node: EnsureKnowledgeDriveNodeRequest,
) -> Result<EnsureDriveWorkspaceNode, KnowledgeDriveWorkspaceError> {
    let logical_path = safe_logical_path(&node.logical_path)
        .map_err(|error| KnowledgeDriveWorkspaceError::InvalidRequest(error.to_string()))?;
    match node.kind {
        EnsureKnowledgeDriveNodeKind::Folder => Ok(EnsureDriveWorkspaceNode::folder(logical_path)),
        EnsureKnowledgeDriveNodeKind::File => {
            let object_ref = node.object_ref.ok_or_else(|| {
                KnowledgeDriveWorkspaceError::InvalidRequest(format!(
                    "object_ref is required for file node: {logical_path}"
                ))
            })?;
            let content_length = i64::try_from(object_ref.size_bytes).map_err(|_| {
                KnowledgeDriveWorkspaceError::InvalidRequest(format!(
                    "size_bytes is out of range for drive file object: {}",
                    object_ref.logical_path
                ))
            })?;
            let checksum_sha256_hex = object_ref.checksum_sha256_hex.ok_or_else(|| {
                KnowledgeDriveWorkspaceError::InvalidRequest(format!(
                    "checksum_sha256_hex is required for drive file object: {}",
                    object_ref.logical_path
                ))
            })?;
            Ok(EnsureDriveWorkspaceNode::file(
                logical_path,
                DriveWorkspaceObjectRef {
                    storage_provider_id: object_ref.storage_provider_id,
                    bucket: object_ref.bucket,
                    object_key: object_ref.object_key,
                    content_type: object_ref.content_type,
                    content_length,
                    checksum_sha256_hex,
                },
            ))
        }
    }
}

fn knowledge_summary_from_drive_node(
    node: DriveWorkspaceNode,
) -> Result<KnowledgeDriveNodeSummary, KnowledgeDriveNodeTreeError> {
    let kind = match node.kind {
        DriveWorkspaceNodeKind::Folder => DriveNodeKind::Folder,
        DriveWorkspaceNodeKind::File => DriveNodeKind::File,
    };
    let size_bytes = node
        .content_length
        .map(|value| {
            u64::try_from(value).map_err(|_| {
                KnowledgeDriveNodeTreeError::Internal(format!(
                    "drive content_length must be non-negative: {value}"
                ))
            })
        })
        .transpose()?;
    let children_count = u64::try_from(node.children_count).map_err(|_| {
        KnowledgeDriveNodeTreeError::Internal(format!(
            "drive children_count must be non-negative: {}",
            node.children_count
        ))
    })?;
    Ok(KnowledgeDriveNodeSummary {
        drive_node_id: node.id,
        parent_drive_node_id: node.parent_node_id,
        kind,
        name: node.name,
        path: node.path,
        content_type: node.content_type,
        size_bytes,
        children_count: Some(children_count),
        updated_at: node.updated_at,
    })
}

fn knowledge_page_from_drive_page(
    page: DriveWorkspaceChildrenPage,
) -> Result<KnowledgeDriveNodePage, KnowledgeDriveNodeTreeError> {
    let nodes = page
        .nodes
        .into_iter()
        .map(knowledge_summary_from_drive_node)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(KnowledgeDriveNodePage {
        nodes,
        next_cursor: page.next_offset.map(|offset| offset.to_string()),
    })
}

fn decode_cursor(cursor: Option<&str>) -> Result<i64, KnowledgeDriveNodeTreeError> {
    let Some(cursor) = cursor else {
        return Ok(0);
    };
    let offset = cursor.trim().parse::<i64>().map_err(|_| {
        KnowledgeDriveNodeTreeError::InvalidRequest("cursor must be a numeric offset".to_string())
    })?;
    if offset < 0 {
        return Err(KnowledgeDriveNodeTreeError::InvalidRequest(
            "cursor must be non-negative".to_string(),
        ));
    }
    Ok(offset)
}

fn checksum_sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn verified_request_checksum(
    request_checksum: Option<&str>,
    computed_checksum: &str,
) -> Result<String, KnowledgeStorageError> {
    let Some(request_checksum) = request_checksum else {
        return Ok(computed_checksum.to_string());
    };
    let normalized = normalize_sha256_hex(request_checksum)?;
    if normalized != computed_checksum {
        return Err(KnowledgeStorageError::IntegrityFailed(
            "checksum_sha256_hex does not match request body".to_string(),
        ));
    }
    Ok(normalized)
}

fn normalize_sha256_hex(value: &str) -> Result<String, KnowledgeStorageError> {
    let checksum = value.trim().to_ascii_lowercase();
    let checksum = checksum.strip_prefix("sha256:").unwrap_or(&checksum);
    if checksum.len() != 64 || !checksum.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(KnowledgeStorageError::InvalidRequest(
            "checksum_sha256_hex must be a 64-character hex SHA-256 digest".to_string(),
        ));
    }
    Ok(checksum.to_string())
}

fn content_version_id(provider_version_id: Option<String>, checksum_sha256_hex: &str) -> String {
    normalized_provider_version_id(provider_version_id)
        .unwrap_or_else(|| synthetic_content_version_id(checksum_sha256_hex))
}

fn content_version_id_from_head(
    provider_version_id: Option<String>,
    checksum_sha256_hex: Option<&str>,
) -> Option<String> {
    normalized_provider_version_id(provider_version_id).or_else(|| {
        checksum_sha256_hex
            .filter(|checksum| !checksum.trim().is_empty())
            .map(synthetic_content_version_id)
    })
}

fn normalized_provider_version_id(provider_version_id: Option<String>) -> Option<String> {
    provider_version_id
        .map(|version_id| version_id.trim().to_string())
        .filter(|version_id| !version_id.is_empty())
}

fn synthetic_content_version_id(checksum_sha256_hex: &str) -> String {
    let checksum = checksum_sha256_hex.trim();
    if checksum.starts_with("sha256:") {
        checksum.to_string()
    } else {
        format!("sha256:{checksum}")
    }
}

fn map_workspace_product_error(error: DriveProductError) -> KnowledgeDriveWorkspaceError {
    match error {
        DriveProductError::Validation(message) | DriveProductError::Conflict(message) => {
            KnowledgeDriveWorkspaceError::InvalidRequest(message)
        }
        DriveProductError::NotFound(message) | DriveProductError::PermissionDenied(message) => {
            KnowledgeDriveWorkspaceError::Upstream(message)
        }
        DriveProductError::Internal(message) => KnowledgeDriveWorkspaceError::Internal(message),
    }
}

fn map_space_product_error(error: DriveProductError) -> KnowledgeDriveSpaceProvisionerError {
    match error {
        DriveProductError::Validation(message) => {
            KnowledgeDriveSpaceProvisionerError::InvalidRequest(message)
        }
        DriveProductError::Conflict(message)
        | DriveProductError::NotFound(message)
        | DriveProductError::PermissionDenied(message) => {
            KnowledgeDriveSpaceProvisionerError::Upstream(message)
        }
        DriveProductError::Internal(message) => {
            KnowledgeDriveSpaceProvisionerError::Internal(message)
        }
    }
}

fn map_tree_product_error(error: DriveProductError) -> KnowledgeDriveNodeTreeError {
    match error {
        DriveProductError::Validation(message) => {
            KnowledgeDriveNodeTreeError::InvalidRequest(message)
        }
        DriveProductError::Conflict(message)
        | DriveProductError::NotFound(message)
        | DriveProductError::PermissionDenied(message) => {
            KnowledgeDriveNodeTreeError::Upstream(message)
        }
        DriveProductError::Internal(message) => KnowledgeDriveNodeTreeError::Internal(message),
    }
}
