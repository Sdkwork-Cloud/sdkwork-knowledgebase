use crate::okf::document::{OkfConceptDocument, OkfDocumentError};
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

pub fn validate_concept_id(concept_id: &str) -> Result<(), OkfConformanceError> {
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
        if segment.is_empty() || segment.len() > 128 {
            return Err(OkfConformanceError::InvalidConceptId(
                "concept id segments must be non-empty and <= 128 chars".to_string(),
            ));
        }
        if !segment
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '_'))
        {
            return Err(OkfConformanceError::InvalidConceptId(
                "concept id segments must match [a-z0-9_-]+".to_string(),
            ));
        }
    }
    Ok(())
}

pub fn validate_concept_document(
    document: &OkfConceptDocument,
    concept_id: &str,
) -> Result<(), OkfConformanceError> {
    validate_concept_id(concept_id)?;
    if document.concept_type.trim().is_empty() {
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
