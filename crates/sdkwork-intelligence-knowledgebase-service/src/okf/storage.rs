use crate::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeStorageError,
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
) -> Result<String, KnowledgeStorageError> {
    let bytes = read_managed_object_bytes(drive, logical_path).await?;
    String::from_utf8(bytes)
        .map_err(|error| KnowledgeStorageError::InvalidRequest(error.to_string()))
}

pub async fn read_managed_object_bytes(
    drive: &dyn KnowledgeDriveStorage,
    logical_path: &str,
) -> Result<Vec<u8>, KnowledgeStorageError> {
    for role in MANAGED_OBJECT_ROLES {
        if let Ok(object_ref) = drive
            .head_object(HeadKnowledgeObjectRequest::managed_artifact(
                logical_path,
                role,
            ))
            .await
        {
            return drive.get_object_bytes(&object_ref).await;
        }
    }
    Err(KnowledgeStorageError::internal(format!(
        "missing okf bundle object at {logical_path}"
    )))
}
