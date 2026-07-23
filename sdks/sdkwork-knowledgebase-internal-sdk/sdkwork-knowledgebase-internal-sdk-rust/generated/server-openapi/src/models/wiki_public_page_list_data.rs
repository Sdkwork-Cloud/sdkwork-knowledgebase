use serde::{Deserialize, Serialize};

use crate::models::{PageInfo, WikiPublicPageMetadata};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WikiPublicPageListData {
    pub items: Vec<WikiPublicPageMetadata>,

    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}
