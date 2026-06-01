use async_trait::async_trait;
use sdkwork_knowledgebase_contract::mirror::{
    DeltaOperations, MirrorContentPolicy, MirrorDatabase,
};
use sdkwork_knowledgebase_product::mirror::{
    KnowledgeLocalMirrorManifestService, KnowledgeLocalMirrorManifestServiceError,
    PersistLocalMirrorDeltaManifestRequest, PersistLocalMirrorSnapshotManifestRequest,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use std::sync::Mutex;

#[tokio::test]
async fn snapshot_manifest_is_projected_to_llm_wiki_local_mirror_and_written_through_drive() {
    let drive = RecordingDrive::default();
    let service = KnowledgeLocalMirrorManifestService::new(&drive);

    let result = service
        .persist_snapshot_manifest(PersistLocalMirrorSnapshotManifestRequest {
            space_id: "space_uuid".to_string(),
            snapshot_version: "2026.06.01.000001".to_string(),
            base_snapshot_version: None,
            created_at: "2026-06-01T00:00:00Z".to_string(),
            content_policy: mirror_policy(),
            database: mirror_database(),
            objects_manifest: "drive_objects/objects_manifest.jsonl".to_string(),
            index_manifests: vec!["indexes/full_text/index_manifest.json".to_string()],
            checksums: "checksums.sha256".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(result.manifest.package_kind, "snapshot");
    assert_eq!(result.manifest.schema_version, "1.0.0");
    assert_eq!(result.manifest.space_id, "space_uuid");
    assert_eq!(
        result.object_ref.logical_path,
        "mirror/snapshots/2026.06.01.000001/mirror_manifest.json"
    );
    assert_eq!(result.object_ref.object_role, "local_mirror_snapshot");
    assert_eq!(
        result.object_ref.content_type,
        "application/json; charset=utf-8"
    );
    assert!(result.object_ref.checksum_sha256_hex.is_some());
    assert_eq!(
        result
            .manifest
            .llm_wiki_compatibility
            .agent_instruction_path,
        "AGENTS.md"
    );
    assert_eq!(
        result.manifest.llm_wiki_compatibility.schema_path,
        "schema/wiki_schema.yaml"
    );
    assert_eq!(result.manifest.llm_wiki_compatibility.raw_root, "raw/");
    assert_eq!(result.manifest.llm_wiki_compatibility.wiki_root, "wiki/");
    assert_eq!(
        result.manifest.llm_wiki_compatibility.index_path,
        "wiki/index.md"
    );
    assert_eq!(
        result.manifest.llm_wiki_compatibility.log_path,
        "wiki/log.md"
    );

    let body = drive.last_body_text();
    assert!(body.contains("\"packageKind\": \"snapshot\""));
    assert!(body.contains("\"agentInstructionPath\": \"AGENTS.md\""));
    assert!(body.contains("\"schemaPath\": \"schema/wiki_schema.yaml\""));
    assert!(body.contains("\"objectsManifest\": \"drive_objects/objects_manifest.jsonl\""));
}

#[tokio::test]
async fn delta_manifest_records_incremental_update_inputs_and_is_written_through_drive() {
    let drive = RecordingDrive::default();
    let service = KnowledgeLocalMirrorManifestService::new(&drive);

    let result = service
        .persist_delta_manifest(PersistLocalMirrorDeltaManifestRequest {
            space_id: "space_uuid".to_string(),
            from_snapshot_version: "2026.06.01.000001".to_string(),
            to_snapshot_version: "2026.06.01.000002".to_string(),
            created_at: "2026-06-01T01:00:00Z".to_string(),
            requires_schema_version: "1.0.0".to_string(),
            operations: DeltaOperations {
                sql_patch: "sql_patch.jsonl".to_string(),
                added_objects: "added_objects.jsonl".to_string(),
                changed_objects: "changed_objects.jsonl".to_string(),
                deleted_objects: "deleted_objects.jsonl".to_string(),
                index_patch: "index_patch/".to_string(),
            },
            checksums: "checksums.sha256".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(result.manifest.package_kind, "delta");
    assert_eq!(result.manifest.schema_version, "1.0.0");
    assert_eq!(
        result.object_ref.logical_path,
        "mirror/deltas/2026.06.01.000001_to_2026.06.01.000002/delta_manifest.json"
    );
    assert_eq!(result.object_ref.object_role, "local_mirror_delta");
    assert_eq!(
        result.manifest.operations.added_objects,
        "added_objects.jsonl"
    );
    assert_eq!(result.manifest.operations.index_patch, "index_patch/");
    assert!(result.object_ref.checksum_sha256_hex.is_some());

    let body = drive.last_body_text();
    assert!(body.contains("\"packageKind\": \"delta\""));
    assert!(body.contains("\"fromSnapshotVersion\": \"2026.06.01.000001\""));
    assert!(body.contains("\"toSnapshotVersion\": \"2026.06.01.000002\""));
}

#[tokio::test]
async fn local_mirror_manifest_service_rejects_unsafe_path_segments() {
    let drive = RecordingDrive::default();
    let service = KnowledgeLocalMirrorManifestService::new(&drive);

    let error = service
        .persist_snapshot_manifest(PersistLocalMirrorSnapshotManifestRequest {
            space_id: "space_uuid".to_string(),
            snapshot_version: "../escape".to_string(),
            base_snapshot_version: None,
            created_at: "2026-06-01T00:00:00Z".to_string(),
            content_policy: mirror_policy(),
            database: mirror_database(),
            objects_manifest: "drive_objects/objects_manifest.jsonl".to_string(),
            index_manifests: vec![],
            checksums: "checksums.sha256".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        KnowledgeLocalMirrorManifestServiceError::InvalidRequest(_)
    ));
    assert_eq!(drive.write_count(), 0);
}

fn mirror_policy() -> MirrorContentPolicy {
    MirrorContentPolicy {
        include_raw_sources: false,
        include_parsed_artifacts: true,
        include_wiki: true,
        include_embeddings: true,
        include_eval_reports: false,
    }
}

fn mirror_database() -> MirrorDatabase {
    MirrorDatabase {
        engine: "sqlite".to_string(),
        schema_version: "1.0.0".to_string(),
        file: "sqlite/knowledgebase.sqlite".to_string(),
        checksum_sha256: "sqlite-checksum".to_string(),
    }
}

#[derive(Default)]
struct RecordingDrive {
    writes: Mutex<Vec<RecordedWrite>>,
}

#[derive(Debug, Clone)]
struct RecordedWrite {
    object_ref: KnowledgeObjectRef,
    body: String,
}

impl RecordingDrive {
    fn last_body_text(&self) -> String {
        self.writes.lock().unwrap().last().unwrap().body.clone()
    }

    fn write_count(&self) -> usize {
        self.writes.lock().unwrap().len()
    }
}

#[async_trait]
impl KnowledgeDriveStorage for RecordingDrive {
    async fn put_object(
        &self,
        request: PutKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        let body = String::from_utf8(request.body.clone())
            .map_err(|error| KnowledgeStorageError::InvalidRequest(error.to_string()))?;
        let object_ref = KnowledgeObjectRef {
            bucket: "knowledgebase-test".to_string(),
            object_key: request.logical_path.clone(),
            logical_path: request.logical_path,
            object_role: request.object_role,
            content_type: request.content_type,
            size_bytes: request.body.len() as u64,
            checksum_sha256_hex: request.checksum_sha256_hex,
            etag: None,
            version_id: None,
        };
        self.writes.lock().unwrap().push(RecordedWrite {
            object_ref: object_ref.clone(),
            body,
        });
        Ok(object_ref)
    }

    async fn head_object(
        &self,
        _request: HeadKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        Err(KnowledgeStorageError::internal("not needed"))
    }

    async fn get_object_text(
        &self,
        object_ref: &KnowledgeObjectRef,
    ) -> Result<String, KnowledgeStorageError> {
        self.writes
            .lock()
            .unwrap()
            .iter()
            .find(|write| write.object_ref.object_key == object_ref.object_key)
            .map(|write| write.body.clone())
            .ok_or_else(|| KnowledgeStorageError::NotFound(object_ref.object_key.clone()))
    }
}
