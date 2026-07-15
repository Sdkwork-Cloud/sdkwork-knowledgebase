use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OkfQualityRun {
    pub id: i64,

    pub state: String,
}
