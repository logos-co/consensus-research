use serde::Serialize;

pub type SerializedNodeState = serde_json::Value;

#[derive(Serialize)]
pub struct OutData {
    pub id: u64,
    pub iteration: u64,
    pub round: u64,
    pub vote: u8,
    pub _type: String,
    pub state: SerializedNodeState,
}

pub trait NodeStateRecord {
    fn get_serialized_state_record(&self) -> SerializedNodeState {
        SerializedNodeState::Null
    }
}
