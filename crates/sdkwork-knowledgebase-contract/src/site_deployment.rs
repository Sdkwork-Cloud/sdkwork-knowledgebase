use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSiteDeploymentRequest {
    pub space_id: u64,
    pub platform: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_domain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site_logo_data_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSiteDeploymentResult {
    pub accepted: bool,
    pub status: String,
    pub deployment_id: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSiteDeploymentPreview {
    pub deployment_id: String,
    pub content_type: String,
    pub html: String,
}
