use sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DRIVE_OBJECT_REF_SNAPSHOT_SCHEMA_VERSION: u8 = 1;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PersistedKnowledgeDriveObjectRef {
    schema_version: u8,
    id: u64,
    space_id: u64,
    drive_space_id: Option<String>,
    drive_node_id: Option<String>,
    logical_path: Option<String>,
    drive_provider_kind: String,
    drive_storage_provider_id: String,
    drive_bucket: String,
    drive_object_key: String,
    drive_object_version: Option<String>,
    drive_etag: Option<String>,
    content_type: Option<String>,
    size_bytes: u64,
    checksum_sha256_hex: Option<String>,
    object_role: String,
    access_mode: String,
}

pub(crate) fn encode_drive_object_ref_snapshot(
    object_ref: &KnowledgeDriveObjectRef,
) -> Result<Value, String> {
    serde_json::to_value(PersistedKnowledgeDriveObjectRef::from(object_ref))
        .map_err(|error| error.to_string())
}

pub(crate) fn decode_drive_object_ref_snapshot(
    value: &Value,
) -> Result<KnowledgeDriveObjectRef, String> {
    let persisted: PersistedKnowledgeDriveObjectRef =
        serde_json::from_value(value.clone()).map_err(|error| error.to_string())?;
    if persisted.schema_version != DRIVE_OBJECT_REF_SNAPSHOT_SCHEMA_VERSION {
        return Err(format!(
            "unsupported drive object ref snapshot schemaVersion {}",
            persisted.schema_version
        ));
    }
    Ok(persisted.into())
}

impl From<&KnowledgeDriveObjectRef> for PersistedKnowledgeDriveObjectRef {
    fn from(value: &KnowledgeDriveObjectRef) -> Self {
        Self {
            schema_version: DRIVE_OBJECT_REF_SNAPSHOT_SCHEMA_VERSION,
            id: value.id,
            space_id: value.space_id,
            drive_space_id: value.drive_space_id.clone(),
            drive_node_id: value.drive_node_id.clone(),
            logical_path: value.logical_path.clone(),
            drive_provider_kind: value.drive_provider_kind.clone(),
            drive_storage_provider_id: value.drive_storage_provider_id.clone(),
            drive_bucket: value.drive_bucket.clone(),
            drive_object_key: value.drive_object_key.clone(),
            drive_object_version: value.drive_object_version.clone(),
            drive_etag: value.drive_etag.clone(),
            content_type: value.content_type.clone(),
            size_bytes: value.size_bytes,
            checksum_sha256_hex: value.checksum_sha256_hex.clone(),
            object_role: value.object_role.clone(),
            access_mode: value.access_mode.clone(),
        }
    }
}

impl From<PersistedKnowledgeDriveObjectRef> for KnowledgeDriveObjectRef {
    fn from(value: PersistedKnowledgeDriveObjectRef) -> Self {
        Self {
            id: value.id,
            space_id: value.space_id,
            drive_space_id: value.drive_space_id,
            drive_node_id: value.drive_node_id,
            logical_path: value.logical_path,
            drive_provider_kind: value.drive_provider_kind,
            drive_storage_provider_id: value.drive_storage_provider_id,
            drive_bucket: value.drive_bucket,
            drive_object_key: value.drive_object_key,
            drive_object_version: value.drive_object_version,
            drive_etag: value.drive_etag,
            content_type: value.content_type,
            size_bytes: value.size_bytes,
            checksum_sha256_hex: value.checksum_sha256_hex,
            object_role: value.object_role,
            access_mode: value.access_mode,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object_ref() -> KnowledgeDriveObjectRef {
        KnowledgeDriveObjectRef {
            id: 91,
            space_id: 7,
            drive_space_id: Some("drv-kb-001".to_string()),
            drive_node_id: Some("node-001".to_string()),
            logical_path: Some("raw/documents/report.md".to_string()),
            drive_provider_kind: "sdkwork-drive".to_string(),
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-source".to_string(),
            drive_object_key: "incoming/quarterly-report.md".to_string(),
            drive_object_version: Some("v1".to_string()),
            drive_etag: Some("etag".to_string()),
            content_type: Some("text/markdown; charset=utf-8".to_string()),
            size_bytes: 128,
            checksum_sha256_hex: Some("abc123".to_string()),
            object_role: "original_document".to_string(),
            access_mode: "managed".to_string(),
        }
    }

    #[test]
    fn private_snapshot_round_trips_every_drive_locator_field() {
        let expected = object_ref();
        let encoded = encode_drive_object_ref_snapshot(&expected).unwrap();

        assert_eq!(encoded["schemaVersion"], 1);
        assert_eq!(encoded["driveProviderKind"], "sdkwork-drive");
        assert_eq!(encoded["driveStorageProviderId"], "provider-kb");
        assert_eq!(encoded["driveBucket"], "knowledgebase-source");
        assert_eq!(encoded["driveObjectKey"], "incoming/quarterly-report.md");
        assert_eq!(
            decode_drive_object_ref_snapshot(&encoded).unwrap(),
            expected
        );
    }

    #[test]
    fn private_snapshot_rejects_unknown_schema_versions() {
        let mut encoded = encode_drive_object_ref_snapshot(&object_ref()).unwrap();
        encoded["schemaVersion"] = Value::from(2);

        assert!(decode_drive_object_ref_snapshot(&encoded)
            .unwrap_err()
            .contains("unsupported drive object ref snapshot schemaVersion 2"));
    }
}
