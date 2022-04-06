extern crate hex;
use actix_cors::Cors;
use actix_web::{
    error, get, http::header, middleware::Logger, post, web, App, Error, HttpRequest, HttpResponse,
    HttpServer,
};
use chrono::Utc;
use env_logger;
use log;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr};

pub async fn build_http_service(
    dapp_port: String,
    http_dispatcher_url: String,
) -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    let addr = SocketAddr::from(([127, 0, 0, 1], dapp_port.parse::<u16>().unwrap()));

    log::debug!("build_http_service");
    HttpServer::new(move || {
        App::new()
            .app_data(http_dispatcher_url.clone())
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

#[get("/inspect/{payload}")]
async fn inspect(payload: web::Path<String>) -> Result<HttpResponse, Error> {
    log::debug!("inspect");

    //#TODO: Need to define a proper Payload struct
    //#TODO: Query Room State by payload
    Ok(HttpResponse::Ok().json(payload.into_inner()))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdvanceStateRequest {
    operation: String,
}

//#TODO: Return the latest GameState so that the client can know how to perform their next step
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdvanceStateResponse {
    result: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
enum GameResult {
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

#[post("/advance")]
async fn advance_state(
    json_req: web::Json<AdvanceStateRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse, Error> {
    log::debug!("advance_state");

    /*
        #Game Flow to implement
        1. FindRoom (Join the only room) -> through /advance route
        2. Get game status -> (through /inspect/{payload} route?)
        3. Action (Paper / Scissors / Stone) -> through /advance route
        4. Repeat by step 2.
    */

    let client = Client::new();
    let http_dispatcher_url =
        http_req
            .app_data::<String>()
            .ok_or(error::ErrorInternalServerError(
                "http_dispatcher_url data not found",
            ))?;

    log::debug!("Call to Http Dispatcher: Adding Notice");
    let mut resp_payload = HashMap::new();

    let response = AdvanceStateResponse {
        result: String::from("Test"),
    };

    let serialized = serde_json::to_string(&response).unwrap();
    log::debug!("Serialized response: {}", serialized);
    let hex_response = hex::encode(serialized);
    log::debug!("Hex-encoded response: {}", hex_response);

    resp_payload.insert("payload", hex_response);

    let notice_resp = client
        .post(format!("{}/notice", http_dispatcher_url))
        .json(&resp_payload)
        .send()
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    let notice_status = notice_resp.status();
    let notice_resp_content = notice_resp
        .text()
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    log::debug!(
        "Received notice status {} body {}",
        notice_status,
        notice_resp_content
    );

    log::debug!("Call to Http Dispatcher: Finishing");
    let mut accept_body = HashMap::new();
    accept_body.insert("status", "accept");

    let finish_resp = client
        .post(format!("{}/finish", http_dispatcher_url))
        .json(&accept_body)
        .send()
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    log::debug!("Received finish status {}", finish_resp.status());

    Ok(HttpResponse::Ok().json(json_req))
}
