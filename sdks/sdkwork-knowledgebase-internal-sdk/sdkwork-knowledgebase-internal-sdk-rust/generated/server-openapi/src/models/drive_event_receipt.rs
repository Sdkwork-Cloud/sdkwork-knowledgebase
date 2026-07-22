use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveEventReceipt {
    #[serde(rename = "eventId")]
    pub event_id: String,

    #[serde(rename = "checkpointId")]
    pub checkpoint_id: String,

    #[serde(rename = "sequenceNo")]
    pub sequence_no: String,

    pub disposition: String,
}
