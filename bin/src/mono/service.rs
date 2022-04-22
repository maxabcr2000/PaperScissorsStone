extern crate hex;
use crate::mono::http_dispatcher;
use crate::mono::model;
use actix_cors::Cors;
use actix_web::{
    error, get, http::header, middleware::Logger, post, web, web::Data, App, Error, HttpRequest,
    HttpResponse, HttpServer, Responder,
};
use chrono::Utc;
use http_dispatcher::{send_finish_request, send_notice};
use model::{
    AdvanceRequest, AdvanceStateResponse, FinishStatus, GameRequest, GameState, GameWinner, Player,
    PlayerAction, RoundResult, PLAYER_BIG_DAMAGE, PLAYER_HP, PLAYER_SMALL_DAMAGE,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Mutex, MutexGuard};

pub async fn build_http_service(
    dapp_port: String,
    http_dispatcher_url: String,
) -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    let addr = SocketAddr::from(([127, 0, 0, 1], dapp_port.parse::<u16>().unwrap()));
    let game = Data::new(Mutex::new(GameState {
        round: 1,
        players: HashMap::<String, Player>::new(),
        round_result: GameWinner::None,
        game_result: GameWinner::None,
    }));

    log::debug!("build_http_service");
    HttpServer::new(move || {
        App::new()
            .app_data(http_dispatcher_url.clone())
            .app_data(Data::clone(&game))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["POST", "GET", "PUT"])
                    .allowed_headers(vec![
                        header::AUTHORIZATION,
                        header::ACCEPT,
                        header::CONTENT_TYPE,
                    ])
                    .max_age(3600),
            )
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(healthz)
            .service(inspect)
            .service(advance_state)
    })
    .bind(addr)?
    .run()
    .await
}

fn service_info() -> String {
    serde_json::json!({
        "name":    env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
        "startTime": Utc::now().timestamp(),
        "server": "game server"
    })
    .to_string()
}

#[get("/healthz")]
async fn healthz() -> Result<HttpResponse, Error> {
    log::debug!("healthz");

    let static_service_info: &str = Box::leak(service_info().into_boxed_str());
    Ok(HttpResponse::Ok()
        .insert_header(header::ContentType::json())
        .body(static_service_info))
}

//#Note: We can query GameState by grqphql directly from Rollup, this Api is only for frontend
#[get("/inspect/{payload}")]
async fn inspect(payload: web::Path<String>) -> Result<impl Responder, Error> {
    log::debug!("inspect");

    //#TODO: Need to define a proper Payload struct
    //#TODO: Query Room State by payload
    Ok(HttpResponse::Ok().json(payload.into_inner()))
}

//#TODO: Define Error messages as const strings in model.rs
#[post("/advance")]
async fn advance_state(
    json_req: web::Json<AdvanceRequest>,
    http_req: HttpRequest,
) -> Result<impl Responder, Error> {
    log::debug!("advance_state");

    let http_dispatcher_url = http_req
        .app_data::<String>()
        .expect("http_dispatcher_url data not found");

    let game = http_req
        .app_data::<Data<Mutex<GameState>>>()
        .expect("game not found");

    let mut mut_game = game.lock().unwrap();
    /*
        Game Flow to implement
        1. FindRoom (Join the only room) -> through /advance route
        2. Get game status -> (through /inspect/{payload} route?)
        3. Action (Paper / Scissors / Stone) -> through /advance route
        4. Repeat by step 2.
    */

    let request = json_req.into_inner();
    let hex_payload = request.payload.trim_start_matches("0x");
    log::debug!("hex_payload: {}", &hex_payload);

    let json_payload = hex::decode(hex_payload).map_err(|e| error::ErrorBadRequest(e))?;
    let game_req: GameRequest = serde_json::from_slice(&json_payload).unwrap();
    let player_name = game_req.player_name;

    //#TODO: Use Enum or Const string in match arms
    let gameflow_result = match game_req.operation.as_str() {
        "find_game" => {
            let new_player = Player {
                name: player_name.clone(),
                hp: PLAYER_HP,
                current_action: None,
            };

            //#TODO: Check GameState.game_result to see if game has already ended, if ended, reset GameState to start a new game

            match mut_game.players.len() {
                0 => {
                    mut_game.players.insert(player_name, new_player);
                    Ok(mut_game)
                }
                1 => {
                    //Check if player is already in game
                    match mut_game.players.get(&player_name) {
                        Some(_) => Err("Already in game"),
                        None => {
                            mut_game.players.insert(player_name, new_player);
                            Ok(mut_game)
                        }
                    }
                }
                _ => Err("Room is full"),
            }
        }
        "paper" => handle_player_action(player_name, PlayerAction::Paper, mut_game),
        "scissors" => handle_player_action(player_name, PlayerAction::Scissors, mut_game),
        "stone" => handle_player_action(player_name, PlayerAction::Stone, mut_game),
        _ => Err("Not supported action"),
    };

    match gameflow_result {
        Ok(v) => {
            //#NOTE: Use deref operator: * to take out the GameState inside the MutexGuard
            let v = &*v;
            let result_json = serde_json::to_string(v).unwrap();
            let response = AdvanceStateResponse {
                result: result_json,
            };

            let notice_payload = serde_json::to_string(&response).unwrap();
            match send_notice(&http_dispatcher_url, notice_payload).await {
                Ok(()) => {
                    log::debug!("send_notice successed");
                    send_finish_request(&http_dispatcher_url, FinishStatus::Accept).await?;
                }
                Err(e) => {
                    log::debug!("Error occurred while send_notice: {}", e);
                    send_finish_request(&http_dispatcher_url, FinishStatus::Reject).await?;
                }
            }
        }
        Err(e) => {
            log::debug!("Error occurred while handling game logic: {}", e);
            send_finish_request(&http_dispatcher_url, FinishStatus::Reject).await?;
        }
    }

    Ok(HttpResponse::Accepted())
}

pub fn handle_player_action(
    player_name: String,
    player_action: PlayerAction,
    mut_game: MutexGuard<GameState>,
) -> Result<MutexGuard<GameState>, &'static str> {
    //Check GameState.game_result to see if game has already ended
    let mut mut_game = mut_game;

    //#TODO: Prevent player from sending duplicate actions

    match mut_game.game_result {
        GameWinner::None => {
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
                            RoundResult::Draw => {
                                mut_player.hp -= PLAYER_SMALL_DAMAGE;
                                mut_other_player.hp -= PLAYER_SMALL_DAMAGE;
                                mut_game.round_result = GameWinner::Draw;
                            }
                            RoundResult::Lose => {
                                mut_player.hp -= PLAYER_BIG_DAMAGE;
                                mut_game.round_result =
                                    GameWinner::Player(mut_other_player.name.clone());
                            }
                            RoundResult::Win => {
                                mut_other_player.hp -= PLAYER_BIG_DAMAGE;
                                mut_game.round_result = GameWinner::Player(player_name.clone());
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

                        //Update changes back to mut_game

                        mut_game
                            .players
                            .insert(mut_other_player.name.clone(), mut_other_player.clone());
                    }
                    mut_game
                        .players
                        .insert(player_name.clone(), mut_player.clone());

                    Ok(mut_game)
                } else {
                    Err("Invalid player number")
                }
            } else {
                Err("Invalid player")
            }
        }
        _ => Err("Game has already ended"),
    }
}

pub fn paper_scissors_stone(my_action: PlayerAction, other_action: PlayerAction) -> RoundResult {
    match my_action {
        PlayerAction::Paper => match other_action {
            PlayerAction::Paper => RoundResult::Draw,
            PlayerAction::Scissors => RoundResult::Lose,
            PlayerAction::Stone => RoundResult::Win,
        },
        PlayerAction::Scissors => match other_action {
            PlayerAction::Paper => RoundResult::Win,
            PlayerAction::Scissors => RoundResult::Draw,
            PlayerAction::Stone => RoundResult::Lose,
        },
        PlayerAction::Stone => match other_action {
            PlayerAction::Paper => RoundResult::Lose,
            PlayerAction::Scissors => RoundResult::Win,
            PlayerAction::Stone => RoundResult::Draw,
        },
    }
}
