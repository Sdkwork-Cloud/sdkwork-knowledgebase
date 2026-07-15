use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OkfCandidateReviewRequest {
    #[serde(rename = "reviewerId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewer_id: Option<i64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}
