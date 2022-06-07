extern crate hex;
use crate::mono::model;
use hyper::{header as HyperHeader, Body, Client, Method, Request, Response};
use model::{FinishStatus, IndexResponse, Notice};

pub async fn send_notice(http_dispatcher_url: &str, payload: String) -> bool {
    log::debug!("Call to Http Dispatcher: Adding Notice");
    let client = Client::new();

    let hexed_payload = hex::encode(payload);
    log::debug!("Hex-encoded payload: {}", hexed_payload);

    let notice = Notice {
        payload: "0x".to_string() + &hexed_payload,
    };

    let notice_json = serde_json::to_string(&notice).unwrap();
    log::debug!("notice_json: {}", notice_json);

    let notice_req = match Request::builder()
        .method(Method::POST)
        .header(HyperHeader::CONTENT_TYPE, "application/json")
        .uri(format!("{}/notice", http_dispatcher_url))
        .body(Body::from(notice_json)) {
            Ok(req) => req,
            Err(e) => {
                log::debug!("Error occurred while building notice request: {}",e);
                return false;
            }
        };

    let notice_resp = match client
        .request(notice_req)
        .await {
            Ok(resp) => resp,
            Err(e) => {
                log::debug!("Error occurred while sending notice: {}", e);
                return false;
            }
        };

    let notice_status = notice_resp.status();
    let bz = match &hyper::body::to_bytes(notice_resp)
    .await {
        Ok(bz) => bz.to_vec(),
        Err(e) => {
            log::debug!("Error occurred while decoding /notice resp: {}", e);
            return false;
        }
    };

    let id_response = match serde_json::from_slice::<IndexResponse>(&bz) {
        Ok(json) => json,
        Err(e) => {
            log::debug!("Error occurred while decoding /notice resp: {}", e);
            return false;
        }
    };

    log::debug!(
        "Received notice status {} body {:?}",
        notice_status,
        &id_response
    );

    true
}

pub async fn send_finish_request(
    http_dispatcher_url: &str,
    status: FinishStatus,
) -> Option<Response<Body>> {
    let client = Client::new();

    let status_value = status.to_string();
    log::debug!("status_value: {}", status_value);

    let mut json_status = std::collections::HashMap::new();
    json_status.insert("status", status_value);

    let finish_req = match Request::builder()
        .method(Method::POST)
        .header(HyperHeader::CONTENT_TYPE, "application/json")
        .uri(format!("{}/finish", http_dispatcher_url))
        .body(Body::from(serde_json::to_string(&json_status).unwrap()))
        {
            Ok(req) => req,
            Err(e) => {
                log::debug!("Error occurred while building send_finish_request: {}", e);
                return None;
            }
        };

    match client
        .request(finish_req)
        .await {
            Ok(resp) => Some(resp),
            Err(e) => {
                log::debug!("Error occurred while send_finish_request: {}", e);
                return None;
            }
        }
}
