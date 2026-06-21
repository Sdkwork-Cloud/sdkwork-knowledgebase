use crate::okf::document::{OkfConceptDocument, OkfDocumentError};
use sdkwork_utils_rust::is_blank;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OkfConformanceError {
    #[error("invalid okf concept id: {0}")]
    InvalidConceptId(String),
    #[error("invalid okf concept document: {0}")]
    InvalidDocument(String),
    #[error(transparent)]
    Document(#[from] OkfDocumentError),
}

fn is_catalog_concept_id_segment(segment: &str) -> bool {
    if segment.is_empty() {
        return false;
    }
    let mut chars = segment.chars();
    let first = chars.next().expect("non-empty segment");
    if !(first.is_ascii_alphanumeric() || first == '_') {
        return false;
    }
    segment
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-'))
}

fn validate_sdkwork_concept_id_segment(segment: &str) -> Result<(), OkfConformanceError> {
    if segment.is_empty() || segment.len() > 128 {
        return Err(OkfConformanceError::InvalidConceptId(
            "concept id segments must be non-empty and <= 128 chars".to_string(),
        ));
    }
    let mut chars = segment.chars();
    let first = chars.next().expect("non-empty segment");
    if !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return Err(OkfConformanceError::InvalidConceptId(
            "concept id segments must start with [a-z0-9]".to_string(),
        ));
    }
    if !segment
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '_'))
    {
        return Err(OkfConformanceError::InvalidConceptId(
            "concept id segments must match [a-z0-9][a-z0-9_-]*".to_string(),
        ));
    }
    Ok(())
}

fn validate_concept_id_path(
    concept_id: &str,
    validate_segment: fn(&str) -> Result<(), OkfConformanceError>,
) -> Result<(), OkfConformanceError> {
    let concept_id = concept_id.trim();
    if concept_id.is_empty() || concept_id.len() > 256 {
        return Err(OkfConformanceError::InvalidConceptId(
            "concept id length must be between 1 and 256".to_string(),
        ));
    }
    if concept_id.starts_with('.') || concept_id.contains("..") {
        return Err(OkfConformanceError::InvalidConceptId(
            "concept id must not traverse parent directories".to_string(),
        ));
    }
    for segment in concept_id.split('/') {
        validate_segment(segment)?;
    }
    Ok(())
}

pub fn validate_concept_id(concept_id: &str) -> Result<(), OkfConformanceError> {
    validate_concept_id_path(concept_id, validate_sdkwork_concept_id_segment)
}

pub fn validate_catalog_concept_id(concept_id: &str) -> Result<(), OkfConformanceError> {
    validate_concept_id_path(concept_id, |segment| {
        if !is_catalog_concept_id_segment(segment) {
            return Err(OkfConformanceError::InvalidConceptId(
                "catalog concept id segments must match [A-Za-z0-9_][A-Za-z0-9_.-]*".to_string(),
            ));
        }
        Ok(())
    })
}

pub fn canonicalize_imported_concept_id(concept_id: &str) -> Result<String, OkfConformanceError> {
    validate_catalog_concept_id(concept_id)?;
    let canonical = concept_id
        .split('/')
        .map(|segment| segment.to_ascii_lowercase().replace('.', "_"))
        .collect::<Vec<_>>()
        .join("/");
    validate_concept_id(&canonical)?;
    Ok(canonical)
}

pub fn validate_concept_document(
    document: &OkfConceptDocument,
    concept_id: &str,
) -> Result<(), OkfConformanceError> {
    validate_concept_id(concept_id)?;
    if is_blank(Some(document.concept_type.as_str())) {
        return Err(OkfConformanceError::InvalidDocument(
            "concept type must not be blank".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_bundle_relative_path(path: &str) -> Result<(), OkfConformanceError> {
    if path == "index.md" || path == "log.md" {
        return Ok(());
    }
    if !path.ends_with(".md") {
        return Err(OkfConformanceError::InvalidDocument(
            "bundle concept paths must end with .md".to_string(),
        ));
    }
    if path.starts_with("schema/") {
        return Err(OkfConformanceError::InvalidDocument(
            "schema directory files are not OKF concepts".to_string(),
        ));
    }
    let concept_id = path.strip_suffix(".md").unwrap_or(path);
    validate_concept_id(concept_id)
}

pub fn validate_catalog_concept_bundle_relative_path(
    path: &str,
) -> Result<(), OkfConformanceError> {
    let normalized = path.trim().replace('\\', "/");
    if normalized == "index.md"
        || normalized == "log.md"
        || normalized.ends_with("/index.md")
        || normalized.ends_with("/log.md")
    {
        return Err(OkfConformanceError::InvalidDocument(
            "index.md and log.md are reserved bundle files, not concepts".to_string(),
        ));
    }
    if !normalized.ends_with(".md") {
        return Err(OkfConformanceError::InvalidDocument(
            "bundle concept paths must end with .md".to_string(),
        ));
    }
    if normalized.starts_with("schema/") {
        return Err(OkfConformanceError::InvalidDocument(
            "schema directory files are not OKF concepts".to_string(),
        ));
    }
    let concept_id = normalized
        .strip_suffix(".md")
        .unwrap_or(normalized.as_str());
    validate_catalog_concept_id(concept_id)
}

pub fn validate_concept_bundle_relative_path(path: &str) -> Result<(), OkfConformanceError> {
    let normalized = path.trim().replace('\\', "/");
    if normalized == "index.md"
        || normalized == "log.md"
        || normalized.ends_with("/index.md")
        || normalized.ends_with("/log.md")
    {
        return Err(OkfConformanceError::InvalidDocument(
            "index.md and log.md are reserved bundle files, not concepts".to_string(),
        ));
    }
    validate_bundle_relative_path(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concept_bundle_path_rejects_reserved_files() {
        assert_eq!(
            validate_concept_bundle_relative_path("index.md"),
            Err(OkfConformanceError::InvalidDocument(
                "index.md and log.md are reserved bundle files, not concepts".to_string()
            ))
        );
        validate_concept_bundle_relative_path("tables/users.md").expect("concept path is valid");
    }

    #[test]
    fn catalog_concept_id_accepts_uppercase_and_dots() {
        validate_catalog_concept_id("Tables/Users").expect("catalog id is valid");
        validate_catalog_concept_id("posts.tag/wiki").expect("catalog id with dots is valid");
    }

    #[test]
    fn canonicalize_imported_concept_id_normalizes_catalog_ids() {
        assert_eq!(
            canonicalize_imported_concept_id("Tables/Users").expect("canonical id"),
            "tables/users"
        );
        assert_eq!(
            canonicalize_imported_concept_id("posts.tag/wiki").expect("canonical id"),
            "posts_tag/wiki"
        );
    }

    #[test]
    fn publish_concept_id_rejects_uppercase() {
        assert!(validate_concept_id("Tables/users").is_err());
    }
}
