use async_trait::async_trait;
use sdkwork_drive_storage_contract::{
    DriveByteRange, DriveObjectLocator, DriveObjectStore, DriveObjectStoreError,
    DriveObjectStoreErrorKind, HeadObjectRequest, PutObjectRequest, ReadObjectRangeRequest,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_node_tree::{
    DriveNodeKind, KnowledgeDriveNodePage, KnowledgeDriveNodeSummary, KnowledgeDriveNodeTree,
    KnowledgeDriveNodeTreeError, ListKnowledgeDriveNodeChildrenRequest,
    ResolveKnowledgeDriveNodePathRequest,
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
use sqlx::{AnyPool, Row};
use std::collections::BTreeMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct KnowledgebaseDriveStorageAdapter {
    store: Arc<dyn DriveObjectStore>,
    bucket: String,
    object_key_root: String,
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
        bucket: impl Into<String>,
        object_key_root: impl Into<String>,
    ) -> Self
    where
        S: DriveObjectStore + 'static,
    {
        Self {
            store,
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
impl KnowledgeDriveStorage for KnowledgebaseDriveStorageAdapter {
    async fn put_object(
        &self,
        request: PutKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        let locator = self.locator_for(&request.logical_path)?;
        let size_bytes = request.body.len() as u64;
        let checksum_sha256_hex = request
            .checksum_sha256_hex
            .clone()
            .unwrap_or_else(|| checksum_sha256_hex(&request.body));
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

        Ok(KnowledgeObjectRef {
            bucket: response.locator.bucket,
            object_key: response.locator.object_key,
            logical_path: request.logical_path,
            object_role: request.object_role,
            content_type: request.content_type,
            size_bytes,
            checksum_sha256_hex: Some(checksum_sha256_hex),
            etag: response.etag,
            version_id: response.version_id,
        })
    }

    async fn head_object(
        &self,
        request: HeadKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
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

        Ok(KnowledgeObjectRef {
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
            version_id: response.version_id,
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

        for node in request.nodes {
            let logical_path = safe_logical_path(&node.logical_path)
                .map_err(|error| KnowledgeDriveWorkspaceError::InvalidRequest(error.to_string()))?;
            self.ensure_single_node(&drive_space_id, &logical_path, node)
                .await?;
        }
        Ok(())
    }
}

impl KnowledgebaseDriveWorkspaceAdapter {
    async fn ensure_single_node(
        &self,
        drive_space_id: &str,
        logical_path: &str,
        node: EnsureKnowledgeDriveNodeRequest,
    ) -> Result<(), KnowledgeDriveWorkspaceError> {
        let (parent_path, node_name) = split_parent_path(logical_path)?;
        let parent_node_id = match parent_path {
            Some(parent_path) => Some(
                self.resolve_path(drive_space_id, parent_path)
                    .await?
                    .ok_or_else(|| {
                        KnowledgeDriveWorkspaceError::InvalidRequest(format!(
                            "parent drive node is missing for logical_path: {logical_path}"
                        ))
                    })?,
            ),
            None => None,
        };

        if let Some(existing) = self
            .find_child_node(drive_space_id, parent_node_id.as_deref(), node_name)
            .await?
        {
            self.ensure_existing_node_matches(&existing, node.kind, logical_path)
                .await?;
            if node.kind == EnsureKnowledgeDriveNodeKind::File {
                let object_ref = node.object_ref.ok_or_else(|| {
                    KnowledgeDriveWorkspaceError::InvalidRequest(format!(
                        "object_ref is required for file node: {logical_path}"
                    ))
                })?;
                self.ensure_file_object(&existing.id, &object_ref).await?;
            }
            return Ok(());
        }

        let node_id = format!("kb-node-{}", Uuid::new_v4());
        let node_type = match node.kind {
            EnsureKnowledgeDriveNodeKind::Folder => "folder",
            EnsureKnowledgeDriveNodeKind::File => "file",
        };
        let content_state = match node.kind {
            EnsureKnowledgeDriveNodeKind::Folder => "empty",
            EnsureKnowledgeDriveNodeKind::File => "ready",
        };
        sqlx::query(
            "INSERT INTO drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', 1, $8, $8)",
        )
        .bind(&node_id)
        .bind(&self.tenant_id)
        .bind(drive_space_id)
        .bind(parent_node_id.as_deref())
        .bind(node_type)
        .bind(node_name)
        .bind(content_state)
        .bind(&self.operator_id)
        .execute(&self.pool)
        .await
        .map_err(workspace_sqlx_error("insert drive_node failed"))?;

        if node.kind == EnsureKnowledgeDriveNodeKind::File {
            let object_ref = node.object_ref.ok_or_else(|| {
                KnowledgeDriveWorkspaceError::InvalidRequest(format!(
                    "object_ref is required for file node: {logical_path}"
                ))
            })?;
            self.ensure_file_object(&node_id, &object_ref).await?;
        }

        Ok(())
    }

    async fn ensure_existing_node_matches(
        &self,
        existing: &DriveNodeRecord,
        expected_kind: EnsureKnowledgeDriveNodeKind,
        logical_path: &str,
    ) -> Result<(), KnowledgeDriveWorkspaceError> {
        let expected_type = match expected_kind {
            EnsureKnowledgeDriveNodeKind::Folder => "folder",
            EnsureKnowledgeDriveNodeKind::File => "file",
        };
        if existing.node_type != expected_type {
            return Err(KnowledgeDriveWorkspaceError::InvalidRequest(format!(
                "drive node kind mismatch for logical_path {logical_path}: expected {expected_type}, found {}",
                existing.node_type
            )));
        }
        Ok(())
    }

    async fn ensure_file_object(
        &self,
        node_id: &str,
        object_ref: &KnowledgeObjectRef,
    ) -> Result<(), KnowledgeDriveWorkspaceError> {
        let checksum = object_ref.checksum_sha256_hex.clone().ok_or_else(|| {
            KnowledgeDriveWorkspaceError::InvalidRequest(format!(
                "checksum_sha256_hex is required for drive file object: {}",
                object_ref.logical_path
            ))
        })?;
        let content_length = i64::try_from(object_ref.size_bytes).map_err(|_| {
            KnowledgeDriveWorkspaceError::InvalidRequest(format!(
                "size_bytes is out of range for drive file object: {}",
                object_ref.logical_path
            ))
        })?;

        let existing: Option<String> = sqlx::query_scalar(
            "SELECT id
             FROM drive_storage_object
             WHERE tenant_id=$1
               AND node_id=$2
               AND bucket=$3
               AND object_key=$4
               AND lifecycle_status='active'
             LIMIT 1",
        )
        .bind(&self.tenant_id)
        .bind(node_id)
        .bind(&object_ref.bucket)
        .bind(&object_ref.object_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(workspace_sqlx_error("query drive_storage_object failed"))?;
        if existing.is_some() {
            return Ok(());
        }

        let next_version_no: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version_no), 0) + 1
             FROM drive_storage_object
             WHERE tenant_id=$1 AND node_id=$2",
        )
        .bind(&self.tenant_id)
        .bind(node_id)
        .fetch_one(&self.pool)
        .await
        .map_err(workspace_sqlx_error(
            "compute drive_storage_object version failed",
        ))?;
        let storage_object_id = format!("kb-object-{}", Uuid::new_v4());

        sqlx::query(
            "INSERT INTO drive_storage_object (
                id, tenant_id, node_id, version_no, bucket, object_key,
                content_type, content_length, checksum_sha256_hex, lifecycle_status,
                created_by, updated_by
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'active', $10, $10)",
        )
        .bind(storage_object_id)
        .bind(&self.tenant_id)
        .bind(node_id)
        .bind(next_version_no)
        .bind(&object_ref.bucket)
        .bind(&object_ref.object_key)
        .bind(&object_ref.content_type)
        .bind(content_length)
        .bind(checksum)
        .bind(&self.operator_id)
        .execute(&self.pool)
        .await
        .map_err(workspace_sqlx_error("insert drive_storage_object failed"))?;

        sqlx::query(
            "UPDATE drive_node
             SET content_state='ready', updated_by=$1, updated_at=CURRENT_TIMESTAMP, version=version + 1
             WHERE tenant_id=$2 AND id=$3 AND lifecycle_status != 'deleted'",
        )
        .bind(&self.operator_id)
        .bind(&self.tenant_id)
        .bind(node_id)
        .execute(&self.pool)
        .await
        .map_err(workspace_sqlx_error(
            "mark drive_node content ready failed",
        ))?;

        Ok(())
    }

    async fn resolve_path(
        &self,
        drive_space_id: &str,
        logical_path: &str,
    ) -> Result<Option<String>, KnowledgeDriveWorkspaceError> {
        let mut parent_node_id: Option<String> = None;
        for segment in logical_path.split('/') {
            let Some(record) = self
                .find_child_node(drive_space_id, parent_node_id.as_deref(), segment)
                .await?
            else {
                return Ok(None);
            };
            parent_node_id = Some(record.id);
        }
        Ok(parent_node_id)
    }

    async fn find_child_node(
        &self,
        drive_space_id: &str,
        parent_node_id: Option<&str>,
        node_name: &str,
    ) -> Result<Option<DriveNodeRecord>, KnowledgeDriveWorkspaceError> {
        let row = sqlx::query(
            "SELECT id, node_type, node_name
             FROM drive_node
             WHERE tenant_id=$1
               AND space_id=$2
               AND lifecycle_status='active'
               AND node_name=$3
               AND ((parent_node_id IS NULL AND $4 IS NULL) OR parent_node_id = $4)
             LIMIT 1",
        )
        .bind(&self.tenant_id)
        .bind(drive_space_id)
        .bind(node_name)
        .bind(parent_node_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(workspace_sqlx_error("query drive_node failed"))?;

        Ok(row.map(|row| DriveNodeRecord {
            id: row.get("id"),
            node_type: row.get("node_type"),
            node_name: row.get("node_name"),
        }))
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
        let mut parent_node_id: Option<String> = None;
        let mut current: Option<KnowledgeDriveNodeSummary> = None;
        for segment in logical_path.split('/') {
            let Some(node) = self
                .find_child_summary(&drive_space_id, parent_node_id.as_deref(), segment, None)
                .await?
            else {
                return Ok(None);
            };
            parent_node_id = Some(node.drive_node_id.clone());
            current = Some(node);
        }
        Ok(current)
    }

    async fn list_children(
        &self,
        request: ListKnowledgeDriveNodeChildrenRequest,
    ) -> Result<KnowledgeDriveNodePage, KnowledgeDriveNodeTreeError> {
        let drive_space_id = safe_drive_id(&request.drive_space_id, "drive_space_id")
            .map_err(|error| KnowledgeDriveNodeTreeError::InvalidRequest(error.to_string()))?;
        let page_size = request.page_size.clamp(1, 200);
        let offset = decode_cursor(request.cursor.as_deref())?;

        let rows = sqlx::query(
            "SELECT
                n.id,
                n.parent_node_id,
                n.node_type,
                n.node_name,
                n.updated_at,
                o.content_type,
                o.content_length,
                (
                    SELECT COUNT(1)
                    FROM drive_node c
                    WHERE c.tenant_id=n.tenant_id
                      AND c.space_id=n.space_id
                      AND c.parent_node_id=n.id
                      AND c.lifecycle_status='active'
                ) AS children_count
             FROM drive_node n
             LEFT JOIN drive_storage_object o
               ON o.tenant_id=n.tenant_id
              AND o.node_id=n.id
              AND o.lifecycle_status='active'
              AND o.version_no=(
                  SELECT MAX(version_no)
                  FROM drive_storage_object latest
                  WHERE latest.tenant_id=n.tenant_id
                    AND latest.node_id=n.id
                    AND latest.lifecycle_status='active'
              )
             WHERE n.tenant_id=$1
               AND n.space_id=$2
               AND n.lifecycle_status='active'
               AND ((n.parent_node_id IS NULL AND $3 IS NULL) OR n.parent_node_id = $3)
             ORDER BY
                CASE n.node_type WHEN 'folder' THEN 0 WHEN 'file' THEN 1 ELSE 2 END,
                n.node_name ASC,
                n.id ASC
             LIMIT $4 OFFSET $5",
        )
        .bind(&self.tenant_id)
        .bind(&drive_space_id)
        .bind(request.parent_drive_node_id.as_deref())
        .bind(i64::from(page_size) + 1)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(node_tree_sqlx_error("list drive_node failed"))?;

        let parent_path = match request.parent_drive_node_id.as_deref() {
            Some(parent_id) => self.resolve_node_path(&drive_space_id, parent_id).await?,
            None => String::new(),
        };
        let mut nodes = rows
            .into_iter()
            .map(|row| summary_from_row(row, &parent_path))
            .collect::<Result<Vec<_>, _>>()?;
        let next_cursor = if nodes.len() > page_size as usize {
            nodes.pop();
            Some((offset + i64::from(page_size)).to_string())
        } else {
            None
        };

        Ok(KnowledgeDriveNodePage { nodes, next_cursor })
    }
}

impl KnowledgebaseDriveNodeTreeAdapter {
    async fn find_child_summary(
        &self,
        drive_space_id: &str,
        parent_node_id: Option<&str>,
        node_name: &str,
        parent_path: Option<&str>,
    ) -> Result<Option<KnowledgeDriveNodeSummary>, KnowledgeDriveNodeTreeError> {
        let row = sqlx::query(
            "SELECT
                n.id,
                n.parent_node_id,
                n.node_type,
                n.node_name,
                n.updated_at,
                o.content_type,
                o.content_length,
                (
                    SELECT COUNT(1)
                    FROM drive_node c
                    WHERE c.tenant_id=n.tenant_id
                      AND c.space_id=n.space_id
                      AND c.parent_node_id=n.id
                      AND c.lifecycle_status='active'
                ) AS children_count
             FROM drive_node n
             LEFT JOIN drive_storage_object o
               ON o.tenant_id=n.tenant_id
              AND o.node_id=n.id
              AND o.lifecycle_status='active'
              AND o.version_no=(
                  SELECT MAX(version_no)
                  FROM drive_storage_object latest
                  WHERE latest.tenant_id=n.tenant_id
                    AND latest.node_id=n.id
                    AND latest.lifecycle_status='active'
              )
             WHERE n.tenant_id=$1
               AND n.space_id=$2
               AND n.lifecycle_status='active'
               AND n.node_name=$3
               AND ((n.parent_node_id IS NULL AND $4 IS NULL) OR n.parent_node_id = $4)
             LIMIT 1",
        )
        .bind(&self.tenant_id)
        .bind(drive_space_id)
        .bind(node_name)
        .bind(parent_node_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(node_tree_sqlx_error("query drive_node failed"))?;

        row.map(|row| summary_from_row(row, parent_path.unwrap_or("")))
            .transpose()
    }

    async fn resolve_node_path(
        &self,
        drive_space_id: &str,
        node_id: &str,
    ) -> Result<String, KnowledgeDriveNodeTreeError> {
        let mut current_node_id = Some(node_id.to_string());
        let mut segments = Vec::new();
        while let Some(id) = current_node_id {
            let row = sqlx::query(
                "SELECT node_name, parent_node_id
                 FROM drive_node
                 WHERE tenant_id=$1
                   AND space_id=$2
                   AND id=$3
                   AND lifecycle_status='active'",
            )
            .bind(&self.tenant_id)
            .bind(drive_space_id)
            .bind(&id)
            .fetch_optional(&self.pool)
            .await
            .map_err(node_tree_sqlx_error("resolve drive_node path failed"))?
            .ok_or_else(|| {
                KnowledgeDriveNodeTreeError::InvalidRequest(format!(
                    "parent drive node is missing: {id}"
                ))
            })?;
            segments.push(row.get::<String, _>("node_name"));
            current_node_id = row.get("parent_node_id");
        }
        segments.reverse();
        Ok(segments.join("/"))
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

#[derive(Debug, Clone)]
struct DriveNodeRecord {
    id: String,
    node_type: String,
    #[allow(dead_code)]
    node_name: String,
}

fn split_parent_path(
    logical_path: &str,
) -> Result<(Option<&str>, &str), KnowledgeDriveWorkspaceError> {
    match logical_path.rsplit_once('/') {
        Some((parent, name)) if !parent.is_empty() && !name.is_empty() => Ok((Some(parent), name)),
        None if !logical_path.is_empty() => Ok((None, logical_path)),
        _ => Err(KnowledgeDriveWorkspaceError::InvalidRequest(format!(
            "invalid logical_path: {logical_path}"
        ))),
    }
}

fn safe_drive_id(value: &str, field_name: &str) -> Result<String, KnowledgeDriveWorkspaceError> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 128
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.')
    {
        return Err(KnowledgeDriveWorkspaceError::InvalidRequest(format!(
            "invalid {field_name}"
        )));
    }
    Ok(value.to_string())
}

fn summary_from_row(
    row: sqlx::any::AnyRow,
    parent_path: &str,
) -> Result<KnowledgeDriveNodeSummary, KnowledgeDriveNodeTreeError> {
    let node_type: String = row.get("node_type");
    let kind = match node_type.as_str() {
        "folder" => DriveNodeKind::Folder,
        "file" => DriveNodeKind::File,
        _ => {
            return Err(KnowledgeDriveNodeTreeError::Internal(format!(
                "unsupported drive node_type: {node_type}"
            )));
        }
    };
    let name: String = row.get("node_name");
    let path = if parent_path.is_empty() {
        name.clone()
    } else {
        format!("{parent_path}/{name}")
    };
    let content_length = row.get::<Option<i64>, _>("content_length");
    let children_count = row.get::<i64, _>("children_count");
    Ok(KnowledgeDriveNodeSummary {
        drive_node_id: row.get("id"),
        parent_drive_node_id: row.get("parent_node_id"),
        kind,
        name,
        path,
        content_type: row.get("content_type"),
        size_bytes: content_length.and_then(|value| u64::try_from(value).ok()),
        children_count: u64::try_from(children_count).ok(),
        updated_at: row.get("updated_at"),
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

fn workspace_sqlx_error(
    context: &'static str,
) -> impl Fn(sqlx::Error) -> KnowledgeDriveWorkspaceError {
    move |error| KnowledgeDriveWorkspaceError::Internal(format!("{context}: {error}"))
}

fn node_tree_sqlx_error(
    context: &'static str,
) -> impl Fn(sqlx::Error) -> KnowledgeDriveNodeTreeError {
    move |error| KnowledgeDriveNodeTreeError::Internal(format!("{context}: {error}"))
}
