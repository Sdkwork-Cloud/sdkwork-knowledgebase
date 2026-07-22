use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WikiPublication {
    #[serde(rename = "publicationUuid")]
    pub publication_uuid: String,

    pub title: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "homepageSourcePath")]
    pub homepage_source_path: String,

    #[serde(rename = "defaultLocale")]
    pub default_locale: String,

    #[serde(rename = "supportedLocales")]
    pub supported_locales: Vec<String>,

    #[serde(rename = "navigationMode")]
    pub navigation_mode: String,

    #[serde(rename = "themeKey")]
    pub theme_key: String,

    #[serde(rename = "themeVersion")]
    pub theme_version: String,

    #[serde(rename = "rendererPolicyVersion")]
    pub renderer_policy_version: String,

    #[serde(rename = "searchEnabled")]
    pub search_enabled: bool,

    #[serde(rename = "robotsPolicy")]
    pub robots_policy: String,

    #[serde(rename = "sitemapEnabled")]
    pub sitemap_enabled: bool,

    #[serde(rename = "providerGeneration")]
    pub provider_generation: String,

    #[serde(rename = "navigationGeneration")]
    pub navigation_generation: String,

    #[serde(rename = "searchGeneration")]
    pub search_generation: String,
}
