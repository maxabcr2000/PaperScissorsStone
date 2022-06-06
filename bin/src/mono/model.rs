use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum_macros::{Display as StrumDisplay, EnumString};

#[derive(Debug, Clone, Deserialize, Serialize, StrumDisplay, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum PlayerAction {
    FindGame,
    Paper,
    Scissors,
    Stone,
}

pub enum RoundResult {
    Win,
    Lose,
    Draw,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum GameWinner {
    Player(String),
    Draw,
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Player {
    //#Note: This will be unique id for now
    pub name: String,
    pub hp: u16,
    pub current_action: Option<PlayerAction>,
}

//#NOTE: When the round_winner is decided, we need to update round number, reset Player status + round_winner, then Add GameState as Notice again
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameState {
    pub round: u32,
    pub players: HashMap<String, Player>,
    pub round_result: GameWinner,
    pub game_result: GameWinner,
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
    /*
        #Note: :
        This is actually the data we passed in through input
        We'll later convert payload into GameReqest object
    */
    pub payload: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameRequest {
    pub operation: String,
    pub player_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdvanceStateResponse {
    pub result: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, StrumDisplay, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum FinishStatus {
    Accept,
    Reject,
}

pub const PLAYER_HP: u16 = 1000;
pub const PLAYER_BIG_DAMAGE: u16 = 300;
pub const PLAYER_SMALL_DAMAGE: u16 = 100;
