use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WikiPage {
    #[serde(rename = "projectionUuid")]
    pub projection_uuid: String,

    #[serde(rename = "canonicalRoute")]
    pub canonical_route: String,

    #[serde(rename = "fileKind")]
    pub file_kind: String,

    #[serde(rename = "mediaType")]
    pub media_type: String,

    #[serde(rename = "sizeBytes")]
    pub size_bytes: String,

    #[serde(rename = "contentSha256")]
    pub content_sha256: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,

    #[serde(rename = "navOrder")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nav_order: Option<i64>,

    #[serde(rename = "pagePublicVersion")]
    pub page_public_version: String,

    #[serde(rename = "publicUpdatedAt")]
    pub public_updated_at: String,
}
