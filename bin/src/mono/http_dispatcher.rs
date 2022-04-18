extern crate hex;
use crate::mono::model;
use actix_web::{error, Error};
use hyper::{header as HyperHeader, Body, Client, Method, Request};
use model::{FinishStatus, IndexResponse, Notice};

pub async fn send_notice(http_dispatcher_url: &str, payload: String) -> Result<(), Error> {
    log::debug!("Call to Http Dispatcher: Adding Notice");
    let client = Client::new();

    let hexed_payload = hex::encode(payload);
    log::debug!("Hex-encoded payload: {}", hexed_payload);

    let notice = Notice {
        payload: "0x".to_string() + &hexed_payload,
    };

    let notice_json = serde_json::to_string(&notice).unwrap();
    log::debug!("notice_json: {}", notice_json);

    let notice_req = Request::builder()
        .method(Method::POST)
        .header(HyperHeader::CONTENT_TYPE, "application/json")
        .uri(format!("{}/notice", http_dispatcher_url))
        .body(Body::from(notice_json))?;

    let notice_resp = client
        .request(notice_req)
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    let notice_status = notice_resp.status();
    let id_response = serde_json::from_slice::<IndexResponse>(
        &hyper::body::to_bytes(notice_resp)
            .await
            .map_err(|e| error::ErrorInternalServerError(e))?
            .to_vec(),
    )
    .map_err(|e| error::ErrorInternalServerError(e))?;

    log::debug!(
        "Received notice status {} body {:?}",
        notice_status,
        &id_response
    );

    Ok(())
}

pub async fn send_finish_request(
    http_dispatcher_url: &str,
    status: FinishStatus,
) -> Result<(), Error> {
    log::debug!("Call to Http Dispatcher: Finishing");
    let client = Client::new();

    let status_value = match status {
        FinishStatus::Accept => "accept",
        FinishStatus::Reject => "reject",
    };

    let mut json_status = std::collections::HashMap::new();
    json_status.insert("status", status_value);

    let finish_req = Request::builder()
        .method(Method::POST)
        .header(HyperHeader::CONTENT_TYPE, "application/json")
        .uri(format!("{}/finish", http_dispatcher_url))
        .body(Body::from(serde_json::to_string(&json_status).unwrap()))?;

    client
        .request(finish_req)
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    Ok(())
}
