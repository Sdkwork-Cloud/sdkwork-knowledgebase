use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OkfFileAnswerRequest {
    pub title: String,

    #[serde(rename = "answerMarkdown")]
    pub answer_markdown: String,
}
