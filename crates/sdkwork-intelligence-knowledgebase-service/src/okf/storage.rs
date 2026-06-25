use crate::ports::knowledge_drive_storage::{
    space_uuid_from_drive_space_id, HeadKnowledgeObjectRequest, KnowledgeDriveStorage,
    KnowledgeStorageError,
};

const MANAGED_OBJECT_ROLES: [&str; 6] = [
    "concept_revision",
    "bundle_index",
    "bundle_log",
    "bundle_profile",
    "original_document",
    "output_export",
];

pub async fn read_managed_markdown(
    drive: &dyn KnowledgeDriveStorage,
    logical_path: &str,
    drive_space_id: Option<&str>,
) -> Result<String, KnowledgeStorageError> {
    let bytes = read_managed_object_bytes(drive, logical_path, drive_space_id).await?;
    String::from_utf8(bytes)
        .map_err(|error| KnowledgeStorageError::InvalidRequest(error.to_string()))
}

pub async fn read_managed_object_bytes(
    drive: &dyn KnowledgeDriveStorage,
    logical_path: &str,
    drive_space_id: Option<&str>,
) -> Result<Vec<u8>, KnowledgeStorageError> {
    let space_uuid = drive_space_id.and_then(space_uuid_from_drive_space_id);
    for role in MANAGED_OBJECT_ROLES {
        let head_request = HeadKnowledgeObjectRequest::managed_artifact(logical_path, role);
        let head_request = match &space_uuid {
            Some(space_uuid) => head_request.with_space_uuid(space_uuid.clone()),
            None => head_request,
        };
        if let Ok(object_ref) = drive.head_object(head_request).await {
            return drive.get_object_bytes(&object_ref).await;
        }
    }
    Err(KnowledgeStorageError::internal(format!(
        "missing okf bundle object at {logical_path}"
    )))
}
