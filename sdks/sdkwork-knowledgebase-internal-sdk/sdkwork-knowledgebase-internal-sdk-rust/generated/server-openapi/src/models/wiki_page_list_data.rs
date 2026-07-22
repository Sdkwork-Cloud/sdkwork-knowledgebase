use serde::{Deserialize, Serialize};

use crate::models::{PageInfo, WikiPage};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WikiPageListData {
    pub items: Vec<WikiPage>,

    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}
