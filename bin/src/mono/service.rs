use actix_cors::Cors;
use actix_web::{
    get, http::header, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, ResponseError,
};
use chrono::Utc;
use log;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr};

pub async fn build_http_service(
    dapp_port: String,
    http_dispatcher_url: String,
) -> std::io::Result<()> {
    let static_service_info: &str = Box::leak(service_info().into_boxed_str());
    let addr = SocketAddr::from(([127, 0, 0, 1], dapp_port.parse::<u16>().unwrap()));

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
            // .route(
            //     "/healthz",
            //     web::get().to(move || {
            //         HttpResponse::Ok()
            //             .insert_header(header::ContentType::json())
            //             .body(static_service_info)
            //     }),
            // )
            .service(inspect)
            .service(advance_state)
    })
    .bind(addr)?
    .run()
    .await
}

fn service_info() -> String {
    log::debug!("HEALTHZ");
    serde_json::json!({
        "name":    env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
        "startTime": Utc::now().timestamp(),
        "server": "game server"
    })
    .to_string()
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

#[post("/advance")]
async fn advance_state(
    http_dispatcher_url: String,
    req: web::Json<AdvanceStateRequest>,
) -> Result<HttpResponse, Error> {
    log::debug!("advance_state");

    //#TODO: Match: JoinGame / Move

    let client = Client::new();

    //#TODO: Post to http_dispatcher /notice
    log::debug!("Call to Http Dispatcher: Adding Notice");
    let mut resp_payload = HashMap::new();
    //#TODO: replace "accept" with result hex
    resp_payload.insert("payload", "accept");

    let notice_resp = client
        .post(format!("{}/notice", http_dispatcher_url))
        .json(&resp_payload)
        .send()
        .await?;

    log::debug!(
        "Received notice status {} body {}",
        notice_resp.status_code(),
        notice_resp.text().await?
    );

    //#TODO: Post to http_dispatcher /finish
    log::debug!("Call to Http Dispatcher: Finishing");
    // response = requests.post(dispatcher_url + "/finish", json={"status": "accept"})
    let mut accept_body = HashMap::new();
    accept_body.insert("status", "accept");

    let finish_resp = client
        .post(format!("{}/finish", http_dispatcher_url))
        .json(&accept_body)
        .send()
        .await?;
    log::debug!("Received finish status {}", finish_resp.status_code());

    Ok(HttpResponse::Ok().json(req))
}
