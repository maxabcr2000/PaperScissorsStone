extern crate hex;
use crate::mono::http_dispatcher;
use crate::mono::model;
use http_dispatcher::{send_finish_request, send_notice};
use model::{
    AdvanceRequest, FinishStatus, GameRequest, GameState, GameWinner, Player,
    PlayerAction, RoundResult, PLAYER_BIG_DAMAGE, PLAYER_HP, PLAYER_SMALL_DAMAGE, RollupResponse, RequestType,
};
use std::collections::HashMap;
use hyper::StatusCode;

pub async fn rollup(
    http_dispatcher_url: &str
) {
    log::debug!("HTTP rollup_server url is {}", http_dispatcher_url);
    
    let mut game =  GameState {
        round: 1,
        players: HashMap::<String, Player>::new(),
        round_result: GameWinner::None,
        game_result: GameWinner::None,
    };

    let mut finish_status = FinishStatus::Accept;
    
    loop {
        log::debug!("Sending finish");
        
        let resp = match send_finish_request(http_dispatcher_url, finish_status.clone()).await {
            Some(resp) => resp,
            None => {
                continue;
            }
        };

        if resp.status() == StatusCode::ACCEPTED {
            log::debug!("No pending rollup request, trying again");
        } else {
            let buf = match hyper::body::to_bytes(resp).await {
                Ok(bz) => bz.to_vec(),
                Err(e) => {
                    log::debug!("Failed to handle /finish response: {}", e);
                    continue;
                }
            };

            let rollup_resp = match serde_json::from_slice::<RollupResponse>(&buf) {
                Ok(json) => json,
                Err(e) => {
                    log::debug!("Failed to deserialize RollupResponse: {}", e);
                    continue;
                }
            };
            let rollup_req: Result<RequestType, strum::ParseError> = rollup_resp.request_type.parse();

            match rollup_req {
                Ok(RequestType::AdvanceState) => {
                    //Now we need to handle error case so that we can send reject status through /finish call
                    finish_status =
                        advance_state(http_dispatcher_url, &mut game, rollup_resp.data,  ).await;
                }
                Ok(RequestType::InspectState) => {
                    //Since we have not implement inspect(), do nothing now
                }
                Err(e) => {
                    log::debug!("Error occurred while handling rollup request: {}", e);
                }
            }
        }
    }
}

//#TODO: Define Error messages as const strings in model.rs
async fn advance_state(
    http_dispatcher_url: &str,
    mut_game: &mut GameState,
    request: AdvanceRequest,
) -> FinishStatus {
    log::debug!("advance_state");

    /*
        Game Flow to implement
        1. FindRoom (Join the only room) -> through /advance route
        2. Get game status -> (through /inspect/{payload} route?)
        3. Action (Paper / Scissors / Stone) -> through /advance route
        4. Repeat by step 2.
    */

    let hex_payload = request.payload.trim_start_matches("0x");
    log::debug!("hex_payload: {}", &hex_payload);

    let json_payload = match hex::decode(hex_payload){
        Ok(json) => json,
        Err(e) => {
            log::debug!("Error occurred while decoding GameRequest: {}", e);
            return FinishStatus::Reject;
        }
    };

    let game_req: GameRequest = serde_json::from_slice(&json_payload).unwrap();
    let player_name = game_req.player_name;

    let player_action: Result<PlayerAction, strum::ParseError>= game_req.operation.parse();

    match player_action {
        Ok(PlayerAction::FindGame) => {
            let new_player = Player {
                name: player_name.clone(),
                hp: PLAYER_HP,
                current_action: None,
            };

            //#TODO: Check GameState.game_result to see if game has already ended, if ended, reset GameState to start a new game
            match mut_game.players.len() {
                0 => {
                    mut_game.players.insert(player_name, new_player);
                    
                    save_gamestate_as_notice(http_dispatcher_url, mut_game).await
                }
                1 => {
                    //Check if player is already in game
                    match mut_game.players.get(&player_name) {
                        Some(_) => {
                            log::debug!("Already in game");
                            FinishStatus::Reject
                        },
                        None => {
                            mut_game.players.insert(player_name, new_player);
                           
                            save_gamestate_as_notice(http_dispatcher_url, mut_game).await
                        }
                    }
                }
                _ => {
                    log::debug!("Room is full");
                    FinishStatus::Reject
                },
            }
        },
        Ok(PlayerAction::Paper) | Ok(PlayerAction::Scissors) | Ok(PlayerAction::Stone) => handle_player_action(http_dispatcher_url, player_name, player_action.unwrap(), mut_game).await,
        Err(_) => {
            log::debug!("Not supported action");
            FinishStatus::Reject
        },
    }

}

pub async fn save_gamestate_as_notice(
    http_dispatcher_url: &str,
    mut_game: &mut GameState,
) -> FinishStatus {
    let notice_payload = serde_json::to_string(mut_game).unwrap();
    match send_notice(http_dispatcher_url, notice_payload).await {
        true => {
            log::debug!("send_notice successed");
            FinishStatus::Accept
        }
        false => {
            FinishStatus::Reject
        }
    }
}

pub async fn handle_player_action(
    http_dispatcher_url: &str,
    player_name: String,
    player_action: PlayerAction,
    mut_game: &mut GameState,
) -> FinishStatus {
    //#TODO: Prevent player from sending duplicate actions

    match mut_game.game_result {
        GameWinner::None => {   //Game is still going
            if let Some(p) = mut_game.players.get(&player_name) {
                if let Some(other_player) = mut_game
                    .players
                    .values()
                    .filter(|x| !x.name.eq(&player_name))
                    .next()
                {
                    //#TODO: When new player action arrived and round_result is not GameResult::None, Round+=1 and reset some of the game states

                    let mut mut_player = p.clone();
                    mut_player.current_action = Some(player_action.clone());
                    let mut mut_other_player = other_player.clone();

                    if let Some(other_action) = other_player.current_action.clone() {
                        match paper_scissors_stone(player_action, other_action) {
                            Some(RoundResult::Draw) => {
                                mut_player.hp -= PLAYER_SMALL_DAMAGE;
                                mut_other_player.hp -= PLAYER_SMALL_DAMAGE;
                                mut_game.round_result = GameWinner::Draw;
                            }
                            Some(RoundResult::Lose) => {
                                mut_player.hp -= PLAYER_BIG_DAMAGE;
                                mut_game.round_result =
                                    GameWinner::Player(mut_other_player.name.clone());
                            }
                            Some(RoundResult::Win) => {
                                mut_other_player.hp -= PLAYER_BIG_DAMAGE;
                                mut_game.round_result = GameWinner::Player(player_name.clone());
                            }
                            None => {
                              log::debug!("Invalid PlayerAction");
                              return FinishStatus::Reject;
                            }
                        }

                        //Check if game is ended
                        let game_winner = match (mut_player.hp, mut_other_player.hp) {
                            (hp, other_hp) if hp <= 0 && other_hp <= 0 => GameWinner::Draw,
                            (hp, _) if hp <= 0 => GameWinner::Player(mut_other_player.name.clone()),
                            (_, other_hp) if other_hp <= 0 => {
                                GameWinner::Player(mut_player.name.clone())
                            }
                            _ => GameWinner::None,
                        };

                        mut_game.game_result = game_winner;
                    }

                    save_gamestate_as_notice(http_dispatcher_url, mut_game).await
                } else {
                    log::debug!("Invalid player number");
                    FinishStatus::Reject
                }
            } else {
                log::debug!("Invalid player");
                FinishStatus::Reject
            }
        }
        _ => { //Game has already ended
            log::debug!("Game has already ended");
            FinishStatus::Reject
        },
    }
}

pub fn paper_scissors_stone(my_action: PlayerAction, other_action: PlayerAction) -> Option<RoundResult> {
    match my_action {
        PlayerAction::Paper => match other_action {
            PlayerAction::Paper => Some(RoundResult::Draw),
            PlayerAction::Scissors => Some(RoundResult::Lose),
            PlayerAction::Stone => Some(RoundResult::Win),
            PlayerAction::FindGame => None,
        },
        PlayerAction::Scissors => match other_action {
            PlayerAction::Paper => Some(RoundResult::Win),
            PlayerAction::Scissors => Some(RoundResult::Draw),
            PlayerAction::Stone => Some(RoundResult::Lose),
            PlayerAction::FindGame => None,
        },
        PlayerAction::Stone => match other_action {
            PlayerAction::Paper => Some(RoundResult::Lose),
            PlayerAction::Scissors => Some(RoundResult::Win),
            PlayerAction::Stone => Some(RoundResult::Draw),
            PlayerAction::FindGame => None,
        },
        PlayerAction::FindGame => None
    }
}
