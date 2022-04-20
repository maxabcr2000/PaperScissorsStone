use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PlayerAction {
    Paper,
    Scissors,
    Stone,
}

// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub enum PlayerStatus {
//     Ready,
//     NotReady,
// }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Player {
    //#Note: This will be unique id for now
    pub name: String,
    pub hp: u16,
    pub current_action: Option<PlayerAction>,
    // status: PlayerStatus,
}

//#NOTE: When the round_winner is decided, we need to update round number, reset Player status + round_winner, then Add GameState as Notice again
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameState {
    pub round: u32,
    pub players: Vec<Player>,
    pub round_winner_name: Option<String>,
    pub game_winner_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexResponse {
    pub index: u64,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdvanceStateResponse {
    pub result: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FinishStatus {
    Accept,
    Reject,
}
