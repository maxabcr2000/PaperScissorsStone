use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum GameResult {
    Ongoing,
    Defeated,
    Victory,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerStatus {
    hp: u16,
    //#TODO: change this to Enum
    action: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameState {
    round: u32,
    player1_status: PlayerStatus,
    player2_status: PlayerStatus,
    round_result: GameResult,
    game_result: GameResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexResponse {
    index: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notice {
    pub payload: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvanceMetadata {
    pub msg_sender: String,
    pub epoch_index: u64,
    pub input_index: u64,
    pub block_number: u64,
    pub time_stamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvanceRequest {
    pub metadata: AdvanceMetadata,
    pub payload: String,
}

// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub struct AdvanceStateRequest {
//     operation: String,
// }

//#TODO: Return the latest GameState so that the client can know how to perform their next step
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdvanceStateResponse {
    pub result: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FinishStatus {
    Accept,
    Reject,
}
