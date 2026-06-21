use crate::okf::document::strip_sdkwork_frontmatter;
use crate::okf::render_index_documents;
use crate::okf::storage::{read_managed_markdown, read_managed_object_bytes};
use crate::ports::knowledge_drive_storage::{
    KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError, PutKnowledgeObjectRequest,
};
use crate::ports::knowledge_okf_concept_store::{
    KnowledgeOkfConceptStore, KnowledgeOkfConceptStoreError,
};
use sdkwork_knowledgebase_contract::okf::OkfBundlePaths;
use sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef;
use thiserror::Error;

const YAML_CONTENT_TYPE: &str = "application/yaml; charset=utf-8";
const EXPORT_OBJECT_ROLE: &str = "output_export";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportOkfBundleRequest {
    pub space_id: u64,
    pub export_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedOkfBundle {
    pub export_root: String,
    pub manifest_path: String,
    pub file_count: u32,
    pub manifest_ref: KnowledgeObjectRef,
}

pub struct OkfBundleExporterService<'a> {
    drive: &'a dyn KnowledgeDriveStorage,
    concept_store: &'a dyn KnowledgeOkfConceptStore,
    source_object_refs: Vec<KnowledgeDriveObjectRef>,
}

impl<'a> OkfBundleExporterService<'a> {
    pub fn new(
        drive: &'a dyn KnowledgeDriveStorage,
        concept_store: &'a dyn KnowledgeOkfConceptStore,
    ) -> Self {
        Self {
            drive,
            concept_store,
            source_object_refs: Vec::new(),
        }
    }

    pub fn with_source_object_refs(
        mut self,
        source_object_refs: Vec<KnowledgeDriveObjectRef>,
    ) -> Self {
        self.source_object_refs = source_object_refs;
        self
    }

    pub async fn export_bundle(
        &self,
        request: ExportOkfBundleRequest,
    ) -> Result<ExportedOkfBundle, OkfBundleExporterError> {
        let export_type = normalize_export_type(&request.export_type)?;
        let export_root = format!("output/exports/{export_type}/{}", request.space_id);
        let paths = OkfBundlePaths::default();
        let mut exported_files = Vec::new();
        let concepts = self
            .concept_store
            .list_concept_summaries(request.space_id)
            .await?;

        exported_files.push(
            export_standard_file(self.drive, &export_root, paths.log_md, "log.md", false).await?,
        );
        exported_files.push(
            export_standard_file(
                self.drive,
                &export_root,
                paths.agents_md,
                "schema/AGENTS.md",
                false,
            )
            .await?,
        );
        exported_files.push(
            export_standard_file(
                self.drive,
                &export_root,
                paths.profile_yaml,
                "schema/okf_profile.yaml",
                false,
            )
            .await?,
        );

        for (bundle_relative_path, markdown) in render_index_documents(&concepts) {
            let export_path = format!("{export_root}/{bundle_relative_path}");
            self.drive
                .put_object(PutKnowledgeObjectRequest::text(
                    export_path.clone(),
                    EXPORT_OBJECT_ROLE,
                    markdown,
                    None,
                ))
                .await?;
            exported_files.push(export_path);
        }

        for concept in concepts {
            let content = read_managed_markdown(self.drive, &concept.logical_path).await?;
            let export_path = format!("{export_root}/{}", concept.bundle_relative_path);
            let body = strip_sdkwork_frontmatter(&content);
            self.drive
                .put_object(PutKnowledgeObjectRequest::text(
                    export_path.clone(),
                    EXPORT_OBJECT_ROLE,
                    body,
                    None,
                ))
                .await?;
            exported_files.push(export_path);
        }

        if export_type == "okf_with_sources" {
            for object_ref in &self.source_object_refs {
                let Some(logical_path) = object_ref.logical_path.as_deref() else {
                    continue;
                };
                let raw_relative_path = logical_path
                    .strip_prefix("sources/raw/")
                    .unwrap_or(logical_path);
                let body = read_managed_object_bytes(self.drive, logical_path).await?;
                let export_path = format!("{export_root}/raw/{raw_relative_path}");
                self.drive
                    .put_object(PutKnowledgeObjectRequest {
                        logical_path: export_path.clone(),
                        object_role: EXPORT_OBJECT_ROLE.to_string(),
                        content_type: object_ref
                            .content_type
                            .clone()
                            .unwrap_or_else(|| "application/octet-stream".to_string()),
                        body,
                        checksum_sha256_hex: object_ref.checksum_sha256_hex.clone(),
                    })
                    .await?;
                exported_files.push(export_path);
            }
        }

        let manifest_path = format!("{export_root}/export_manifest.yaml");
        let manifest_files = exported_files
            .iter()
            .map(|path| bundle_relative_path_from_export(&export_root, path))
            .collect::<Vec<_>>();
        let manifest_body = render_export_manifest_yaml(export_type, &manifest_files);
        let manifest_ref = self
            .drive
            .put_object(PutKnowledgeObjectRequest {
                logical_path: manifest_path.clone(),
                object_role: EXPORT_OBJECT_ROLE.to_string(),
                content_type: YAML_CONTENT_TYPE.to_string(),
                body: manifest_body.into_bytes(),
                checksum_sha256_hex: None,
            })
            .await?;

        Ok(ExportedOkfBundle {
            export_root,
            manifest_path,
            file_count: exported_files.len() as u32,
            manifest_ref,
        })
    }
}

fn render_export_manifest_yaml(export_type: &str, files: &[String]) -> String {
    let sources_root = if export_type == "okf_with_sources" {
        "sourcesRoot: \"raw\"\n"
    } else {
        ""
    };
    format!(
        r#"okfVersion: "0.1"
exportType: "{export_type}"
bundleRoot: "."
standardFiles:
  index: "index.md"
  log: "log.md"
  agentInstructions: "schema/AGENTS.md"
  profile: "schema/okf_profile.yaml"
{sources_root}files:
{file_lines}"#,
        file_lines = files
            .iter()
            .map(|path| format!("  - \"{path}\""))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn bundle_relative_path_from_export(export_root: &str, export_path: &str) -> String {
    let normalized_root = export_root.trim().replace('\\', "/");
    let normalized_path = export_path.trim().replace('\\', "/");
    normalized_path
        .strip_prefix(&format!("{normalized_root}/"))
        .or_else(|| normalized_path.strip_prefix(&normalized_root))
        .unwrap_or(normalized_path.as_str())
        .trim_start_matches('/')
        .to_string()
}

async fn export_standard_file(
    drive: &dyn KnowledgeDriveStorage,
    export_root: &str,
    source_logical_path: &str,
    bundle_relative_path: &str,
    strip_sdkwork: bool,
) -> Result<String, OkfBundleExporterError> {
    let content = read_managed_markdown(drive, source_logical_path).await?;
    let body = if strip_sdkwork {
        strip_sdkwork_frontmatter(&content)
    } else {
        content
    };
    let export_path = format!("{export_root}/{bundle_relative_path}");
    drive
        .put_object(PutKnowledgeObjectRequest::text(
            export_path.clone(),
            EXPORT_OBJECT_ROLE,
            body,
            None,
        ))
        .await?;
    Ok(export_path)
}

fn normalize_export_type(export_type: &str) -> Result<&'static str, OkfBundleExporterError> {
    match export_type.trim() {
        "okf_strict" | "snapshot" => Ok("okf_strict"),
        "okf_with_sources" => Ok("okf_with_sources"),
        other if other.is_empty() => Err(OkfBundleExporterError::InvalidRequest(
            "export_type must not be blank".to_string(),
        )),
        other => Err(OkfBundleExporterError::InvalidRequest(format!(
            "unsupported export_type: {other}"
        ))),
    }
}

#[derive(Debug, Error)]
pub enum OkfBundleExporterError {
    #[error("invalid okf bundle export request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    ConceptStore(#[from] KnowledgeOkfConceptStoreError),
    #[error(transparent)]
    Storage(#[from] KnowledgeStorageError),
    #[error("okf bundle exporter internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_manifest_yaml_matches_okf_bundle_compatibility_shape() {
        let yaml = render_export_manifest_yaml("okf_strict", &["index.md".to_string()]);
        assert!(yaml.contains("okfVersion: \"0.1\""));
        assert!(yaml.contains("bundleRoot: \".\""));
        assert!(yaml.contains("profile: \"schema/okf_profile.yaml\""));
        assert!(yaml.contains("  - \"index.md\""));
        assert!(!yaml.contains("output/exports"));
        assert!(!yaml.contains("sourcesRoot"));

        let with_sources =
            render_export_manifest_yaml("okf_with_sources", &["raw/source.bin".to_string()]);
        assert!(with_sources.contains("sourcesRoot: \"raw\""));
    }

    #[test]
    fn bundle_relative_path_from_export_strips_export_root_prefix() {
        assert_eq!(
            bundle_relative_path_from_export(
                "output/exports/okf_strict/7",
                "output/exports/okf_strict/7/tables/users.md"
            ),
            "tables/users.md"
        );
    }
}
