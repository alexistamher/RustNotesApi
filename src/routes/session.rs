use std::collections::HashMap;

use actix_web::{
    HttpRequest, HttpResponse, get,
    web::{self},
};

use crate::util::session_manager::{self, CACHE};
use rust_cipher_lib::crypto_manager;

pub fn session_config(cfg: &mut web::ServiceConfig) {
    cfg.service(exchange).service(validate);
}

#[get("session/exchange")]
async fn exchange(req: HttpRequest) -> HttpResponse {
    let client_public_key = match req.headers().get("x-public-key") {
        Some(key) => key.to_owned().to_str().unwrap().to_owned(),
        None => {
            return HttpResponse::BadRequest().body("public key not present");
        }
    };
    let result = crypto_manager::key_exchange(client_public_key);
    match result {
        Ok(key_exchange_result) => {
            {
                let session_info = key_exchange_result.session_info.clone();
                let sessions_locked = CACHE.sessions.lock();
                let mut sessions = sessions_locked.unwrap();
                sessions.insert(
                    key_exchange_result.session_info.session_id.clone(),
                    session_info,
                );
            }
            let response = HashMap::from([
                (
                    "session_id".to_string(),
                    key_exchange_result.session_info.session_id,
                ),
                ("public_key".to_string(), key_exchange_result.public_key),
                (
                    "shared_secret".to_string(),
                    key_exchange_result.shared_secret,
                ),
            ]);
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("session/validate")]
async fn validate(req: HttpRequest) -> HttpResponse {
    let Some(session_key_header) = req.headers().get("x-session-key") else {
        return HttpResponse::BadRequest().body("session key not present");
    };
    let session_key: &str = session_key_header.to_str().unwrap();
    match session_manager::check_session(session_key) {
        Some(_) => HttpResponse::Ok().finish(),
        None => HttpResponse::Forbidden().finish(),
    }
}
