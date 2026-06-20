use crate::okf::concept_service::{OkfConceptService, OkfConceptServiceError};
use crate::okf::document::parse_okf_markdown;
use crate::okf::storage::read_managed_markdown;
use crate::okf::validator::{validate_concept_bundle_relative_path, validate_concept_document};
use crate::ports::knowledge_drive_storage::KnowledgeDriveStorage;
use sdkwork_knowledgebase_contract::okf::{
    KnowledgeOkfConceptPublication, OkfBundlePaths, PublishKnowledgeOkfConceptRequest,
};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportOkfBundleFile {
    pub bundle_relative_path: String,
    pub markdown: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportOkfBundleRequest {
    pub space_id: u64,
    pub actor: String,
    pub publish: bool,
    pub files: Vec<ImportOkfBundleFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportOkfBundleResult {
    pub imported_concept_count: u32,
    pub skipped_files: Vec<String>,
    pub publications: Vec<KnowledgeOkfConceptPublication>,
}

pub struct OkfBundleImporterService<'a> {
    concept_service: OkfConceptService<'a>,
}

impl<'a> OkfBundleImporterService<'a> {
    pub fn new(concept_service: OkfConceptService<'a>) -> Self {
        Self { concept_service }
    }

    pub async fn import_bundle(
        &self,
        request: ImportOkfBundleRequest,
        drive_space_id: Option<&str>,
    ) -> Result<ImportOkfBundleResult, OkfBundleImporterError> {
        if request.actor.trim().is_empty() {
            return Err(OkfBundleImporterError::InvalidRequest(
                "actor must not be blank".to_string(),
            ));
        }
        if request.files.is_empty() {
            return Err(OkfBundleImporterError::InvalidRequest(
                "at least one bundle file is required".to_string(),
            ));
        }

        let mut publications = Vec::new();
        let mut skipped_files = Vec::new();
        let mut conformance_errors = Vec::new();

        for file in request.files {
            let bundle_relative_path = file.bundle_relative_path.trim().replace('\\', "/");
            if is_reserved_bundle_file(&bundle_relative_path) {
                skipped_files.push(bundle_relative_path);
                continue;
            }
            if let Err(error) = validate_concept_bundle_relative_path(&bundle_relative_path) {
                conformance_errors.push(format!("{bundle_relative_path}: {error}"));
                continue;
            }
            let concept_id = bundle_relative_path
                .strip_suffix(".md")
                .unwrap_or(bundle_relative_path.as_str())
                .to_string();
            let document = match parse_okf_markdown(&file.markdown) {
                Ok(Some(document)) => document,
                Ok(None) => {
                    conformance_errors.push(format!(
                        "{bundle_relative_path}: missing YAML frontmatter with type"
                    ));
                    continue;
                }
                Err(error) => {
                    conformance_errors.push(format!("{bundle_relative_path}: {error}"));
                    continue;
                }
            };
            if let Err(error) = validate_concept_document(&document, &concept_id) {
                conformance_errors.push(format!("{concept_id}: {error}"));
                continue;
            }

            let title = document
                .title
                .clone()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| title_from_concept_id(&concept_id));
            let description = document.description.clone().unwrap_or_default();
            let publication = if request.publish {
                self.concept_service
                    .publish_concept(
                        PublishKnowledgeOkfConceptRequest {
                            space_id: request.space_id,
                            concept_id,
                            title,
                            concept_type: document.concept_type,
                            description,
                            markdown: document.body,
                            source_count: 0,
                            tags: document.tags,
                            actor: request.actor.clone(),
                        },
                        drive_space_id,
                    )
                    .await?
            } else {
                self.concept_service
                    .stage_concept_candidate(
                        PublishKnowledgeOkfConceptRequest {
                            space_id: request.space_id,
                            concept_id,
                            title,
                            concept_type: document.concept_type,
                            description,
                            markdown: document.body,
                            source_count: 0,
                            tags: document.tags,
                            actor: request.actor.clone(),
                        },
                        drive_space_id,
                    )
                    .await?
            };
            publications.push(publication);
        }

        if !conformance_errors.is_empty() {
            return Err(OkfBundleImporterError::Conformance(format!(
                "bundle import rejected due to conformance violations: {}",
                conformance_errors.join("; ")
            )));
        }

        Ok(ImportOkfBundleResult {
            imported_concept_count: publications.len() as u32,
            skipped_files,
            publications,
        })
    }
}

fn is_reserved_bundle_file(bundle_relative_path: &str) -> bool {
    let normalized = bundle_relative_path.trim().replace('\\', "/");
    if normalized == "index.md"
        || normalized.ends_with("/index.md")
        || normalized == "log.md"
        || normalized.ends_with("/log.md")
    {
        return true;
    }
    matches!(
        normalized.as_str(),
        "schema/AGENTS.md" | "schema/okf_profile.yaml"
    ) || normalized.starts_with("schema/")
}

fn title_from_concept_id(concept_id: &str) -> String {
    concept_id
        .rsplit('/')
        .next()
        .unwrap_or(concept_id)
        .replace(['-', '_'], " ")
}

pub fn bundle_relative_path_from_logical_path(logical_path: &str) -> Option<String> {
    let path = logical_path.trim().replace('\\', "/");
    let path = path.strip_prefix("okf/")?;
    Some(path.to_string())
}

pub fn concept_id_from_bundle_relative_path(bundle_relative_path: &str) -> Option<String> {
    OkfBundlePaths::concept_id_from_logical_path(&format!(
        "okf/{}",
        bundle_relative_path
            .strip_suffix(".md")
            .unwrap_or(bundle_relative_path)
    ))
}

pub fn discover_bundle_files_from_directory(
    root: &Path,
) -> Result<Vec<ImportOkfBundleFile>, std::io::Error> {
    let mut files = Vec::new();
    discover_bundle_files_from_directory_inner(root, root, &mut files)?;
    files.sort_by(|left, right| left.bundle_relative_path.cmp(&right.bundle_relative_path));
    Ok(files)
}

fn discover_bundle_files_from_directory_inner(
    root: &Path,
    current: &Path,
    files: &mut Vec<ImportOkfBundleFile>,
) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            discover_bundle_files_from_directory_inner(root, &path, files)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let bundle_relative_path = path
            .strip_prefix(root)
            .map_err(|error| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, error.to_string())
            })?
            .to_string_lossy()
            .replace('\\', "/");
        let markdown = std::fs::read_to_string(&path)?;
        files.push(ImportOkfBundleFile {
            bundle_relative_path,
            markdown,
        });
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportManifest {
    files: Vec<String>,
}

pub async fn load_import_bundle_from_drive(
    drive: &dyn KnowledgeDriveStorage,
    space_id: u64,
) -> Result<Vec<ImportOkfBundleFile>, OkfBundleImporterError> {
    let import_root = format!("input/imports/{space_id}");
    let manifest_path = format!("{import_root}/import_manifest.json");
    let manifest_body = read_managed_markdown(drive, &manifest_path)
        .await
        .map_err(|error| {
            OkfBundleImporterError::InvalidRequest(format!(
                "missing import manifest at {manifest_path}: {error}"
            ))
        })?;
    let manifest: ImportManifest = serde_json::from_str(&manifest_body).map_err(|error| {
        OkfBundleImporterError::InvalidRequest(format!(
            "invalid import manifest at {manifest_path}: {error}"
        ))
    })?;

    let mut files = Vec::new();
    for bundle_relative_path in manifest.files {
        let normalized = bundle_relative_path.trim().replace('\\', "/");
        if is_reserved_bundle_file(&normalized) || !normalized.ends_with(".md") {
            continue;
        }
        let drive_path = format!("{import_root}/{normalized}");
        let markdown = read_managed_markdown(drive, &drive_path)
            .await
            .map_err(|error| {
                OkfBundleImporterError::InvalidRequest(format!(
                    "failed to read import file at {drive_path}: {error}"
                ))
            })?;
        files.push(ImportOkfBundleFile {
            bundle_relative_path: normalized,
            markdown,
        });
    }

    if files.is_empty() {
        return Err(OkfBundleImporterError::InvalidRequest(
            "import manifest did not reference any concept markdown files".to_string(),
        ));
    }
    Ok(files)
}

pub fn stackoverflow_bundle_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../external/knowledge-catalog/okf/bundles/stackoverflow")
}

#[derive(Debug, Error)]
pub enum OkfBundleImporterError {
    #[error("invalid okf bundle import request: {0}")]
    InvalidRequest(String),
    #[error("okf bundle import conformance failed: {0}")]
    Conformance(String),
    #[error(transparent)]
    Storage(#[from] crate::ports::knowledge_drive_storage::KnowledgeStorageError),
    #[error(transparent)]
    ConceptService(#[from] OkfConceptServiceError),
}
