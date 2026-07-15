use serde::{Deserialize, Serialize};

use crate::models::{OkfCandidateResult};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OkfCandidateResultList {
    pub items: Vec<OkfCandidateResult>,
}
