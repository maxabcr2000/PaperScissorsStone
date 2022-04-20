// extern crate hex;
use crate::mono::http_dispatcher;
use crate::mono::model;
use actix_cors::Cors;
use actix_web::{
    get, http::header, middleware::Logger, post, web, App, Error, HttpRequest, HttpResponse,
    HttpServer, Responder,
};
use chrono::Utc;
use env_logger;
use http_dispatcher::{send_finish_request, send_notice};
use log;
use model::{AdvanceRequest, AdvanceStateResponse, FinishStatus, GameState};
use std::net::SocketAddr;

pub async fn build_http_service(
    dapp_port: String,
    http_dispatcher_url: String,
) -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    let addr = SocketAddr::from(([127, 0, 0, 1], dapp_port.parse::<u16>().unwrap()));
    let game = GameState {
        round: 1,
        players: Vec::new(),
        round_winner_name: None,
        game_winner_name: None,
    };

    log::debug!("build_http_service");
    HttpServer::new(move || {
        App::new()
            .app_data(http_dispatcher_url.clone())
            .app_data(game.clone())
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

//#Note: We can query GameState by commmand directly from Rollup?
#[get("/inspect/{payload}")]
async fn inspect(payload: web::Path<String>) -> Result<impl Responder, Error> {
    log::debug!("inspect");

    //#TODO: Need to define a proper Payload struct
    //#TODO: Query Room State by payload
    Ok(HttpResponse::Ok().json(payload.into_inner()))
}

#[post("/advance")]
async fn advance_state(
    json_req: web::Json<AdvanceRequest>,
    http_req: HttpRequest,
) -> Result<impl Responder, Error> {
    log::debug!("advance_state");

    let http_dispatcher_url = http_req
        .app_data::<String>()
        .expect("http_dispatcher_url data not found");

    let game = http_req.app_data::<GameState>().expect("game not found");

    /*
        Game Flow to implement
        1. FindRoom (Join the only room) -> through /advance route
        2. Get game status -> (through /inspect/{payload} route?)
        3. Action (Paper / Scissors / Stone) -> through /advance route
        4. Repeat by step 2.
    */

    //#TODO: Implement each actions
    let gameflow_result = match json_req.payload.as_str() {
        "find_game" => Ok(game),
        "paper" => Ok(game),
        "scissors" => Ok(game),
        "stone" => Ok(game),
        _ => Err("Not supported action"),
    };

    match gameflow_result {
        Ok(v) => {
            let result_json = serde_json::to_string(v).unwrap();
            let response = AdvanceStateResponse {
                result: result_json,
            };

            let notice_payload = serde_json::to_string(&response).unwrap();
            match send_notice(&http_dispatcher_url, notice_payload).await {
                Ok(()) => {
                    log::debug!("send_notice successed");
                    send_finish_request(&http_dispatcher_url, FinishStatus::Accept).await?;

                    Ok(HttpResponse::Accepted())
                }
                Err(e) => {
                    log::debug!("Error occurred while send_notice: {}", e);
                    send_finish_request(&http_dispatcher_url, FinishStatus::Reject).await?;
                    Ok(HttpResponse::InternalServerError())
                }
            }
        }
        Err(e) => {
            log::debug!("Error occurred while handling game logic: {}", e);
            Ok(HttpResponse::InternalServerError())
        }
    }
}
