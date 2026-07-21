use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::serde_int64::{
    deserialize_option_u64_from_string_or_number, deserialize_u64_from_string_or_number,
    serialize_option_u64_as_string, serialize_u64_as_string,
};

macro_rules! string_enum {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($variant),+ }

        impl $name {
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $value),+ }
            }
        }

        impl FromStr for $name {
            type Err = ();

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value { $($value => Ok(Self::$variant),)+ _ => Err(()) }
            }
        }
    };
}

string_enum!(KnowledgeSiteVisibility {
    Private => "private",
    Unlisted => "unlisted",
    Public => "public",
});

string_enum!(KnowledgeSitePublishMode {
    Manual => "manual",
    Automatic => "automatic",
});

string_enum!(KnowledgeSiteState {
    Draft => "draft",
    Active => "active",
    Paused => "paused",
});

string_enum!(KnowledgeSiteReleaseState {
    Building => "building",
    Ready => "ready",
    Failed => "failed",
});

string_enum!(KnowledgeSiteHostBindingType {
    SystemId => "system_id",
    CustomPrefix => "custom_prefix",
    ExternalDomain => "external_domain",
});

string_enum!(KnowledgeSiteHostBindingState {
    Pending => "pending",
    Verified => "verified",
    Active => "active",
    Failed => "failed",
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSite {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub id: u64,
    pub uuid: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub organization_id: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub space_id: u64,
    pub title: String,
    pub visibility: KnowledgeSiteVisibility,
    pub homepage_concept_id: Option<String>,
    pub theme_id: String,
    pub publish_mode: KnowledgeSitePublishMode,
    pub lifecycle_state: KnowledgeSiteState,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub canonical_host_binding_id: Option<u64>,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub current_release_id: Option<u64>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertKnowledgeSiteRequest {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub space_id: u64,
    pub title: String,
    pub visibility: KnowledgeSiteVisibility,
    pub homepage_concept_id: Option<String>,
    pub theme_id: String,
    pub publish_mode: KnowledgeSitePublishMode,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub expected_version: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishKnowledgeSiteReleaseRequest {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub expected_site_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackKnowledgeSiteReleaseRequest {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub release_id: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub expected_site_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSiteRelease {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub id: u64,
    pub uuid: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub site_id: u64,
    pub lifecycle_state: KnowledgeSiteReleaseState,
    pub source_content_hash: String,
    pub manifest_drive_uri: Option<String>,
    pub manifest_drive_space_id: Option<String>,
    pub manifest_drive_node_id: Option<String>,
    pub manifest_checksum_sha256_hex: Option<String>,
    pub page_count: u32,
    pub asset_count: u32,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub previous_release_id: Option<u64>,
    pub error_code: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSiteReleaseList {
    pub items: Vec<KnowledgeSiteRelease>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKnowledgeSiteHostBindingRequest {
    pub binding_type: KnowledgeSiteHostBindingType,
    pub host: String,
    pub canonical: bool,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub expected_site_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSiteHostBinding {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub id: u64,
    pub uuid: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub site_id: u64,
    pub binding_type: KnowledgeSiteHostBindingType,
    pub normalized_host: String,
    pub canonical: bool,
    pub lifecycle_state: KnowledgeSiteHostBindingState,
    pub web_server_site_id: Option<String>,
    pub web_server_domain_id: Option<String>,
    pub web_server_deployment_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSiteHostBindingList {
    pub items: Vec<KnowledgeSiteHostBinding>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSitePublicationResult {
    pub site: KnowledgeSite,
    pub release: KnowledgeSiteRelease,
    pub public_url: String,
}

