use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OkfCandidateResult {
    pub id: i64,

    pub state: String,
}
