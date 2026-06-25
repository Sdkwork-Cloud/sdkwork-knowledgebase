use crate::ingest::OKF_IMPORT_CONCURRENCY;
use crate::okf::concept_service::{
    OkfConceptService, OkfConceptServiceError, OkfPublishConceptOptions,
};
use crate::okf::document::{parse_okf_markdown, OkfConceptDocument};
use crate::okf::storage::{read_managed_markdown, read_managed_object_bytes};
use crate::okf::validator::{
    canonicalize_imported_concept_id, validate_catalog_concept_bundle_relative_path,
    validate_concept_document,
};
use crate::ports::knowledge_drive_storage::{KnowledgeDriveStorage, PutKnowledgeObjectRequest};
use sdkwork_knowledgebase_contract::okf::{
    KnowledgeOkfConceptPublication, OkfBundlePaths, PublishKnowledgeOkfConceptRequest,
};
use sdkwork_knowledgebase_observability::record_okf_bundle_imported;
use sdkwork_utils_rust::is_blank;
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
        if is_blank(Some(request.actor.as_str())) {
            return Err(OkfBundleImporterError::InvalidRequest(
                "actor must not be blank".to_string(),
            ));
        }
        if request.files.is_empty() {
            return Err(OkfBundleImporterError::InvalidRequest(
                "at least one bundle file is required".to_string(),
            ));
        }

        let mut skipped_files = Vec::new();
        let mut conformance_errors = Vec::new();
        let mut publish_requests = Vec::new();

        for file in request.files {
            match classify_bundle_file(file, request.space_id, &request.actor) {
                BundleFileClassification::Skipped(path) => skipped_files.push(path),
                BundleFileClassification::ConformanceViolation(message) => {
                    conformance_errors.push(message)
                }
                BundleFileClassification::Ready(publish_request) => {
                    publish_requests.push(*publish_request)
                }
            }
        }

        if !conformance_errors.is_empty() {
            return Err(OkfBundleImporterError::Conformance(format!(
                "bundle import rejected due to conformance violations: {}",
                conformance_errors.join("; ")
            )));
        }

        let mut publications = Vec::with_capacity(publish_requests.len());
        for batch in publish_requests.chunks(OKF_IMPORT_CONCURRENCY) {
            publications.extend(
                publish_okf_concept_batch(
                    &self.concept_service,
                    batch,
                    request.publish,
                    drive_space_id,
                )
                .await?,
            );
        }

        if request.publish && !publications.is_empty() {
            self.concept_service
                .rebuild_bundle_standard_files(request.space_id, drive_space_id)
                .await?;
        }

        let imported_concept_count = publications.len() as u32;
        record_okf_bundle_imported(request.space_id, imported_concept_count, &request.actor);

        Ok(ImportOkfBundleResult {
            imported_concept_count,
            skipped_files,
            publications,
        })
    }
}

async fn publish_okf_concept_batch(
    concept_service: &OkfConceptService<'_>,
    batch: &[PublishKnowledgeOkfConceptRequest],
    publish: bool,
    drive_space_id: Option<&str>,
) -> Result<Vec<KnowledgeOkfConceptPublication>, OkfBundleImporterError> {
    match batch.len() {
        0 => Ok(vec![]),
        1 => {
            let publication = publish_or_stage_concept(
                concept_service,
                batch[0].clone(),
                publish,
                drive_space_id,
            )
            .await?;
            Ok(vec![publication])
        }
        2 => {
            let (left, right) = tokio::try_join!(
                publish_or_stage_concept(
                    concept_service,
                    batch[0].clone(),
                    publish,
                    drive_space_id,
                ),
                publish_or_stage_concept(
                    concept_service,
                    batch[1].clone(),
                    publish,
                    drive_space_id,
                ),
            )?;
            Ok(vec![left, right])
        }
        3 => {
            let (left, middle, right) = tokio::try_join!(
                publish_or_stage_concept(
                    concept_service,
                    batch[0].clone(),
                    publish,
                    drive_space_id,
                ),
                publish_or_stage_concept(
                    concept_service,
                    batch[1].clone(),
                    publish,
                    drive_space_id,
                ),
                publish_or_stage_concept(
                    concept_service,
                    batch[2].clone(),
                    publish,
                    drive_space_id,
                ),
            )?;
            Ok(vec![left, middle, right])
        }
        _ => {
            let (one, two, three, four) = tokio::try_join!(
                publish_or_stage_concept(
                    concept_service,
                    batch[0].clone(),
                    publish,
                    drive_space_id,
                ),
                publish_or_stage_concept(
                    concept_service,
                    batch[1].clone(),
                    publish,
                    drive_space_id,
                ),
                publish_or_stage_concept(
                    concept_service,
                    batch[2].clone(),
                    publish,
                    drive_space_id,
                ),
                publish_or_stage_concept(
                    concept_service,
                    batch[3].clone(),
                    publish,
                    drive_space_id,
                ),
            )?;
            Ok(vec![one, two, three, four])
        }
    }
}

async fn publish_or_stage_concept(
    concept_service: &OkfConceptService<'_>,
    publish_request: PublishKnowledgeOkfConceptRequest,
    publish: bool,
    drive_space_id: Option<&str>,
) -> Result<KnowledgeOkfConceptPublication, OkfBundleImporterError> {
    if publish {
        concept_service
            .publish_concept_with_options(
                publish_request,
                drive_space_id,
                OkfPublishConceptOptions::bundle_import_batch(),
            )
            .await
            .map_err(OkfBundleImporterError::from)
    } else {
        concept_service
            .stage_concept_candidate(publish_request, drive_space_id)
            .await
            .map_err(OkfBundleImporterError::from)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BundleFileClassification {
    Skipped(String),
    ConformanceViolation(String),
    Ready(Box<PublishKnowledgeOkfConceptRequest>),
}

fn classify_bundle_file(
    file: ImportOkfBundleFile,
    space_id: u64,
    actor: &str,
) -> BundleFileClassification {
    let bundle_relative_path = file.bundle_relative_path.trim().replace('\\', "/");
    if is_reserved_bundle_file(&bundle_relative_path) {
        return BundleFileClassification::Skipped(bundle_relative_path);
    }
    if let Err(error) = validate_catalog_concept_bundle_relative_path(&bundle_relative_path) {
        return BundleFileClassification::ConformanceViolation(format!(
            "{bundle_relative_path}: {error}"
        ));
    }
    let raw_concept_id = bundle_relative_path
        .strip_suffix(".md")
        .unwrap_or(bundle_relative_path.as_str());
    let concept_id = match canonicalize_imported_concept_id(raw_concept_id) {
        Ok(value) => value,
        Err(error) => {
            return BundleFileClassification::ConformanceViolation(format!(
                "{bundle_relative_path}: {error}"
            ))
        }
    };
    let document = match parse_okf_markdown(&file.markdown) {
        Ok(Some(document)) => document,
        Ok(None) => {
            return BundleFileClassification::ConformanceViolation(format!(
                "{bundle_relative_path}: missing YAML frontmatter with type"
            ))
        }
        Err(error) => {
            return BundleFileClassification::ConformanceViolation(format!(
                "{bundle_relative_path}: {error}"
            ))
        }
    };
    if let Err(error) = validate_concept_document(&document, &concept_id) {
        return BundleFileClassification::ConformanceViolation(format!("{concept_id}: {error}"));
    }

    let title = document
        .title
        .clone()
        .filter(|value| !is_blank(Some(value.as_str())))
        .unwrap_or_else(|| title_from_concept_id(&concept_id));
    BundleFileClassification::Ready(Box::new(publish_request_from_document(
        space_id,
        concept_id,
        title,
        document,
        actor.to_string(),
    )))
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

fn publish_request_from_document(
    space_id: u64,
    concept_id: String,
    title: String,
    document: OkfConceptDocument,
    actor: String,
) -> PublishKnowledgeOkfConceptRequest {
    PublishKnowledgeOkfConceptRequest {
        space_id,
        concept_id,
        title,
        concept_type: document.concept_type,
        description: document.description.unwrap_or_default(),
        markdown: document.body,
        source_count: 0,
        tags: document.tags,
        actor,
        resource: document.resource,
        timestamp: document.timestamp,
    }
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

pub fn drive_import_root(space_id: u64, import_id: Option<&str>) -> String {
    let import_key = import_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| space_id.to_string());
    format!("inbox/drive-imports/{import_key}")
}

pub async fn load_import_bundle_from_drive(
    drive: &dyn KnowledgeDriveStorage,
    space_id: u64,
    import_id: Option<&str>,
    drive_space_id: Option<&str>,
) -> Result<Vec<ImportOkfBundleFile>, OkfBundleImporterError> {
    let import_root = drive_import_root(space_id, import_id);
    let (manifest_path, manifest_files) =
        read_import_manifest_files(drive, &import_root, drive_space_id).await?;

    let mut files = Vec::new();
    for batch in manifest_files.chunks(OKF_IMPORT_CONCURRENCY) {
        files.extend(
            read_import_bundle_file_batch(drive, &import_root, drive_space_id, batch).await?,
        );
    }

    if files.is_empty() {
        return Err(OkfBundleImporterError::InvalidRequest(format!(
            "import manifest at {manifest_path} did not reference any concept markdown files"
        )));
    }
    Ok(files)
}

async fn read_import_bundle_file_batch(
    drive: &dyn KnowledgeDriveStorage,
    import_root: &str,
    drive_space_id: Option<&str>,
    batch: &[String],
) -> Result<Vec<ImportOkfBundleFile>, OkfBundleImporterError> {
    let targets = batch
        .iter()
        .filter_map(|bundle_relative_path| {
            let normalized = bundle_relative_path.trim().replace('\\', "/");
            if is_reserved_bundle_file(&normalized) || !normalized.ends_with(".md") {
                return None;
            }
            let drive_path = format!("{import_root}/{normalized}");
            Some((normalized, drive_path))
        })
        .collect::<Vec<_>>();

    match targets.len() {
        0 => Ok(vec![]),
        1 => {
            let (normalized, drive_path) = &targets[0];
            Ok(vec![
                read_import_bundle_markdown_file(drive, normalized, drive_path, drive_space_id)
                    .await?,
            ])
        }
        2 => {
            let (left_path, right_path) = (&targets[0], &targets[1]);
            let (left, right) = tokio::try_join!(
                read_import_bundle_markdown_file(drive, &left_path.0, &left_path.1, drive_space_id,),
                read_import_bundle_markdown_file(
                    drive,
                    &right_path.0,
                    &right_path.1,
                    drive_space_id,
                ),
            )?;
            Ok(vec![left, right])
        }
        3 => {
            let (a, b, c) = (&targets[0], &targets[1], &targets[2]);
            let (left, middle, right) = tokio::try_join!(
                read_import_bundle_markdown_file(drive, &a.0, &a.1, drive_space_id),
                read_import_bundle_markdown_file(drive, &b.0, &b.1, drive_space_id),
                read_import_bundle_markdown_file(drive, &c.0, &c.1, drive_space_id),
            )?;
            Ok(vec![left, middle, right])
        }
        _ => {
            let (a, b, c, d) = (&targets[0], &targets[1], &targets[2], &targets[3]);
            let (one, two, three, four) = tokio::try_join!(
                read_import_bundle_markdown_file(drive, &a.0, &a.1, drive_space_id),
                read_import_bundle_markdown_file(drive, &b.0, &b.1, drive_space_id),
                read_import_bundle_markdown_file(drive, &c.0, &c.1, drive_space_id),
                read_import_bundle_markdown_file(drive, &d.0, &d.1, drive_space_id),
            )?;
            Ok(vec![one, two, three, four])
        }
    }
}

async fn read_import_bundle_markdown_file(
    drive: &dyn KnowledgeDriveStorage,
    bundle_relative_path: &str,
    drive_path: &str,
    drive_space_id: Option<&str>,
) -> Result<ImportOkfBundleFile, OkfBundleImporterError> {
    let markdown = read_managed_markdown(drive, drive_path, drive_space_id)
        .await
        .map_err(|error| {
            OkfBundleImporterError::InvalidRequest(format!(
                "failed to read import file at {drive_path}: {error}"
            ))
        })?;
    Ok(ImportOkfBundleFile {
        bundle_relative_path: bundle_relative_path.to_string(),
        markdown,
    })
}

async fn read_import_manifest_files(
    drive: &dyn KnowledgeDriveStorage,
    import_root: &str,
    drive_space_id: Option<&str>,
) -> Result<(String, Vec<String>), OkfBundleImporterError> {
    const MANIFEST_CANDIDATES: &[&str] = &[
        "import_manifest.yaml",
        "import_manifest.json",
        "export_manifest.yaml",
    ];

    for manifest_name in MANIFEST_CANDIDATES {
        let manifest_path = format!("{import_root}/{manifest_name}");
        let manifest_body = match read_managed_markdown(drive, &manifest_path, drive_space_id).await
        {
            Ok(body) => body,
            Err(_) => continue,
        };
        let files = parse_bundle_manifest_files(&manifest_body).map_err(|error| {
            OkfBundleImporterError::InvalidRequest(format!(
                "invalid import manifest at {manifest_path}: {error}"
            ))
        })?;
        if files.is_empty() {
            return Err(OkfBundleImporterError::InvalidRequest(format!(
                "import manifest at {manifest_path} did not list any bundle files"
            )));
        }
        return Ok((manifest_path, files));
    }

    Err(OkfBundleImporterError::InvalidRequest(format!(
        "missing import manifest under {import_root}; expected import_manifest.yaml, import_manifest.json, or export_manifest.yaml"
    )))
}

fn parse_bundle_manifest_files(body: &str) -> Result<Vec<String>, String> {
    if let Ok(manifest) = serde_json::from_str::<ImportManifest>(body) {
        return Ok(normalize_manifest_file_paths(manifest.files));
    }

    parse_yaml_manifest_files(body)
}

fn parse_yaml_manifest_files(body: &str) -> Result<Vec<String>, String> {
    let mut in_files_section = false;
    let mut files = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "files:" {
            in_files_section = true;
            continue;
        }
        if !in_files_section {
            continue;
        }
        if !trimmed.starts_with('-') {
            if trimmed.ends_with(':') {
                break;
            }
            continue;
        }
        let value = trimmed.trim_start_matches('-').trim();
        if value.is_empty() {
            continue;
        }
        files.push(unquote_manifest_path(value));
    }

    if files.is_empty() {
        return Err("yaml manifest is missing a files list".to_string());
    }
    Ok(normalize_manifest_file_paths(files))
}

fn normalize_manifest_file_paths(paths: Vec<String>) -> Vec<String> {
    paths
        .into_iter()
        .map(|path| path.trim().replace('\\', "/"))
        .filter(|path| !path.is_empty())
        .collect()
}

fn unquote_manifest_path(value: &str) -> String {
    let trimmed = value.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

const STAGED_IMPORT_OBJECT_ROLE: &str = "output_export";

pub async fn stage_export_bundle_for_drive_import(
    drive: &dyn KnowledgeDriveStorage,
    export_root: &str,
    space_id: u64,
    import_id: &str,
    drive_space_id: Option<&str>,
) -> Result<String, OkfBundleImporterError> {
    let (manifest_path, manifest_files) =
        read_export_manifest_files(drive, export_root, drive_space_id).await?;
    let import_root = drive_import_root(space_id, Some(import_id));
    for bundle_relative_path in manifest_files {
        let normalized = bundle_relative_path.trim().replace('\\', "/");
        if normalized.is_empty() {
            continue;
        }
        let source_path = format!("{export_root}/{normalized}");
        let bytes = read_managed_object_bytes(drive, &source_path, drive_space_id)
            .await
            .map_err(OkfBundleImporterError::Storage)?;
        let target_path = format!("{import_root}/{normalized}");
        drive
            .put_object(
                PutKnowledgeObjectRequest {
                    logical_path: target_path,
                    object_role: STAGED_IMPORT_OBJECT_ROLE.to_string(),
                    content_type: content_type_for_bundle_path(&normalized),
                    body: bytes,
                    checksum_sha256_hex: None,
                    space_uuid: None,
                }
                .with_drive_space_id(drive_space_id),
            )
            .await?;
    }
    let manifest_body = read_managed_markdown(drive, &manifest_path, drive_space_id)
        .await
        .map_err(OkfBundleImporterError::Storage)?;
    drive
        .put_object(
            PutKnowledgeObjectRequest {
                logical_path: format!("{import_root}/import_manifest.yaml"),
                object_role: STAGED_IMPORT_OBJECT_ROLE.to_string(),
                content_type: "application/yaml; charset=utf-8".to_string(),
                body: manifest_body.into_bytes(),
                checksum_sha256_hex: None,
                space_uuid: None,
            }
            .with_drive_space_id(drive_space_id),
        )
        .await?;
    Ok(import_root)
}

async fn read_export_manifest_files(
    drive: &dyn KnowledgeDriveStorage,
    export_root: &str,
    drive_space_id: Option<&str>,
) -> Result<(String, Vec<String>), OkfBundleImporterError> {
    const MANIFEST_CANDIDATES: &[&str] = &[
        "export_manifest.yaml",
        "import_manifest.yaml",
        "import_manifest.json",
    ];

    for manifest_name in MANIFEST_CANDIDATES {
        let manifest_path = format!("{export_root}/{manifest_name}");
        let manifest_body = match read_managed_markdown(drive, &manifest_path, drive_space_id).await
        {
            Ok(body) => body,
            Err(_) => continue,
        };
        let files = parse_bundle_manifest_files(&manifest_body).map_err(|error| {
            OkfBundleImporterError::InvalidRequest(format!(
                "invalid export manifest at {manifest_path}: {error}"
            ))
        })?;
        if !files.is_empty() {
            return Ok((manifest_path, files));
        }
    }

    Err(OkfBundleImporterError::InvalidRequest(format!(
        "missing export manifest under {export_root}"
    )))
}

fn content_type_for_bundle_path(path: &str) -> String {
    if path.ends_with(".md") {
        "text/markdown; charset=utf-8".to_string()
    } else if path.ends_with(".yaml") || path.ends_with(".yml") {
        "application/yaml; charset=utf-8".to_string()
    } else if path.ends_with(".json") {
        "application/json; charset=utf-8".to_string()
    } else {
        "application/octet-stream".to_string()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drive_import_root_uses_import_id_when_present() {
        assert_eq!(
            drive_import_root(42, Some("batch-001")),
            "inbox/drive-imports/batch-001"
        );
        assert_eq!(drive_import_root(42, None), "inbox/drive-imports/42");
        assert_eq!(drive_import_root(42, Some("  ")), "inbox/drive-imports/42");
    }

    #[test]
    fn parse_yaml_manifest_files_reads_export_manifest_shape() {
        let yaml = r#"okfVersion: "0.1"
exportType: "okf_strict"
files:
  - "index.md"
  - "tables/users.md"
"#;
        let files = parse_yaml_manifest_files(yaml).expect("yaml manifest");
        assert_eq!(
            files,
            vec!["index.md".to_string(), "tables/users.md".to_string()]
        );
    }

    #[test]
    fn parse_bundle_manifest_files_accepts_json_manifest() {
        let json = r#"{"files":["entities/a.md"]}"#;
        let files = parse_bundle_manifest_files(json).expect("json manifest");
        assert_eq!(files, vec!["entities/a.md".to_string()]);
    }
}
