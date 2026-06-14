use crate::ports::knowledge_drive_storage::{
    KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError, PutKnowledgeObjectRequest,
};
use sdkwork_knowledgebase_contract::{
    mirror::{
        DeltaManifest, DeltaOperations, LlmWikiCompatibility, MirrorContentPolicy, MirrorDatabase,
        MirrorManifest,
    },
    wiki::LlmWikiPaths,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

const LOCAL_MIRROR_SCHEMA_VERSION: &str = "1.0.0";
const LLM_WIKI_PROFILE: &str = "docs/llm-wiki.md";
const SNAPSHOT_PACKAGE_KIND: &str = "snapshot";
const DELTA_PACKAGE_KIND: &str = "delta";
const SNAPSHOT_OBJECT_ROLE: &str = "local_mirror_snapshot";
const DELTA_OBJECT_ROLE: &str = "local_mirror_delta";
const JSON_CONTENT_TYPE: &str = "application/json; charset=utf-8";

pub struct KnowledgeLocalMirrorManifestService<'a> {
    drive: &'a dyn KnowledgeDriveStorage,
}

impl<'a> KnowledgeLocalMirrorManifestService<'a> {
    pub fn new(drive: &'a dyn KnowledgeDriveStorage) -> Self {
        Self { drive }
    }

    pub async fn persist_snapshot_manifest(
        &self,
        request: PersistLocalMirrorSnapshotManifestRequest,
    ) -> Result<PersistedLocalMirrorSnapshotManifest, KnowledgeLocalMirrorManifestServiceError>
    {
        validate_required("space_id", &request.space_id)?;
        validate_required("created_at", &request.created_at)?;
        validate_required("objects_manifest", &request.objects_manifest)?;
        validate_required("checksums", &request.checksums)?;
        let snapshot_version = safe_path_segment("snapshot_version", &request.snapshot_version)?;
        if let Some(base_snapshot_version) = &request.base_snapshot_version {
            safe_path_segment("base_snapshot_version", base_snapshot_version)?;
        }

        let manifest = MirrorManifest {
            schema_version: LOCAL_MIRROR_SCHEMA_VERSION.to_string(),
            space_id: request.space_id,
            snapshot_version: request.snapshot_version,
            base_snapshot_version: request.base_snapshot_version,
            created_at: request.created_at,
            package_kind: SNAPSHOT_PACKAGE_KIND.to_string(),
            content_policy: request.content_policy,
            llm_wiki_compatibility: llm_wiki_compatibility(),
            database: request.database,
            objects_manifest: request.objects_manifest,
            index_manifests: request.index_manifests,
            checksums: request.checksums,
        };

        let object_ref = self
            .persist_json(
                format!("mirror/snapshots/{snapshot_version}/mirror_manifest.json"),
                SNAPSHOT_OBJECT_ROLE,
                &manifest,
            )
            .await?;

        Ok(PersistedLocalMirrorSnapshotManifest {
            manifest,
            object_ref,
        })
    }

    pub async fn persist_delta_manifest(
        &self,
        request: PersistLocalMirrorDeltaManifestRequest,
    ) -> Result<PersistedLocalMirrorDeltaManifest, KnowledgeLocalMirrorManifestServiceError> {
        validate_required("space_id", &request.space_id)?;
        validate_required("created_at", &request.created_at)?;
        validate_required("requires_schema_version", &request.requires_schema_version)?;
        validate_required("checksums", &request.checksums)?;
        let from_snapshot_version =
            safe_path_segment("from_snapshot_version", &request.from_snapshot_version)?;
        let to_snapshot_version =
            safe_path_segment("to_snapshot_version", &request.to_snapshot_version)?;
        if from_snapshot_version == to_snapshot_version {
            return Err(KnowledgeLocalMirrorManifestServiceError::InvalidRequest(
                "from_snapshot_version and to_snapshot_version must differ".to_string(),
            ));
        }

        let manifest = DeltaManifest {
            schema_version: LOCAL_MIRROR_SCHEMA_VERSION.to_string(),
            space_id: request.space_id,
            package_kind: DELTA_PACKAGE_KIND.to_string(),
            from_snapshot_version: request.from_snapshot_version,
            to_snapshot_version: request.to_snapshot_version,
            created_at: request.created_at,
            requires_schema_version: request.requires_schema_version,
            operations: request.operations,
            checksums: request.checksums,
        };

        let object_ref = self
            .persist_json(
                format!(
                    "mirror/deltas/{from_snapshot_version}_to_{to_snapshot_version}/delta_manifest.json"
                ),
                DELTA_OBJECT_ROLE,
                &manifest,
            )
            .await?;

        Ok(PersistedLocalMirrorDeltaManifest {
            manifest,
            object_ref,
        })
    }

    async fn persist_json<T>(
        &self,
        logical_path: String,
        object_role: &str,
        value: &T,
    ) -> Result<KnowledgeObjectRef, KnowledgeLocalMirrorManifestServiceError>
    where
        T: Serialize,
    {
        let body = serde_json::to_vec_pretty(value).map_err(|error| {
            KnowledgeLocalMirrorManifestServiceError::Internal(error.to_string())
        })?;
        let checksum_sha256_hex = checksum_sha256_hex(&body);
        self.drive
            .put_object(PutKnowledgeObjectRequest {
                logical_path,
                object_role: object_role.to_string(),
                content_type: JSON_CONTENT_TYPE.to_string(),
                body,
                checksum_sha256_hex: Some(checksum_sha256_hex),
            })
            .await
            .map_err(KnowledgeLocalMirrorManifestServiceError::Storage)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistLocalMirrorSnapshotManifestRequest {
    pub space_id: String,
    pub snapshot_version: String,
    pub base_snapshot_version: Option<String>,
    pub created_at: String,
    pub content_policy: MirrorContentPolicy,
    pub database: MirrorDatabase,
    pub objects_manifest: String,
    pub index_manifests: Vec<String>,
    pub checksums: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedLocalMirrorSnapshotManifest {
    pub manifest: MirrorManifest,
    pub object_ref: KnowledgeObjectRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistLocalMirrorDeltaManifestRequest {
    pub space_id: String,
    pub from_snapshot_version: String,
    pub to_snapshot_version: String,
    pub created_at: String,
    pub requires_schema_version: String,
    pub operations: DeltaOperations,
    pub checksums: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedLocalMirrorDeltaManifest {
    pub manifest: DeltaManifest,
    pub object_ref: KnowledgeObjectRef,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeLocalMirrorManifestServiceError {
    #[error("invalid local mirror manifest request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Storage(#[from] KnowledgeStorageError),
    #[error("local mirror manifest internal error: {0}")]
    Internal(String),
}

fn llm_wiki_compatibility() -> LlmWikiCompatibility {
    let paths = LlmWikiPaths::default();
    LlmWikiCompatibility {
        profile: LLM_WIKI_PROFILE.to_string(),
        agent_instruction_path: paths.local_mirror_agents_md.to_string(),
        schema_path: format!("{}wiki_schema.yaml", paths.local_mirror_schema_root),
        raw_root: paths.local_mirror_raw_root.to_string(),
        wiki_root: paths.local_mirror_wiki_root.to_string(),
        index_path: format!("{}index.md", paths.local_mirror_wiki_root),
        log_path: format!("{}log.md", paths.local_mirror_wiki_root),
    }
}

fn validate_required(
    field: &str,
    value: &str,
) -> Result<(), KnowledgeLocalMirrorManifestServiceError> {
    if value.trim().is_empty() {
        return Err(KnowledgeLocalMirrorManifestServiceError::InvalidRequest(
            format!("{field} is required"),
        ));
    }
    Ok(())
}

fn safe_path_segment(
    field: &str,
    value: &str,
) -> Result<String, KnowledgeLocalMirrorManifestServiceError> {
    validate_required(field, value)?;
    if value == "."
        || value == ".."
        || value.contains('/')
        || value.contains('\\')
        || value.contains(':')
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_')
    {
        return Err(KnowledgeLocalMirrorManifestServiceError::InvalidRequest(
            format!("{field} is not a safe path segment"),
        ));
    }
    Ok(value.to_string())
}

fn checksum_sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}
