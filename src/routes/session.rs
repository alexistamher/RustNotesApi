use std::collections::HashMap;

use actix_web::{ get, post, web::{ self, Json }, HttpRequest, HttpResponse };

use base64::{ Engine as _, engine::general_purpose::STANDARD as BASE64 };

use crate::{ routes::util::ResultToHttp, util::{ crypto_manager, session_manager } };

pub fn session_config(cfg: &mut web::ServiceConfig) {
    cfg.service(exchange).service(validate).service(test_cipher);
}

#[get("session/exchange")]
async fn exchange(req: HttpRequest) -> HttpResponse {
    let client_public_key = match req.headers().get("x-public-key") {
        Some(key) => key.to_owned().to_str().unwrap().to_owned(),
        None => {
            return HttpResponse::BadRequest().body("public key not present");
        }
    };
    crypto_manager::key_exchange(client_public_key).to_http()
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

#[post("session/test")]
async fn test_cipher(body: Json<HashMap<String, String>>, req: HttpRequest) -> HttpResponse {
    dbg!(&body);
    let Some(session_id_header) = req.headers().get("x-session-key") else {
        return HttpResponse::BadRequest().body("session key not present");
    };
    let Some(nonce_header) = req.headers().get("x-nonce") else {
        return HttpResponse::BadRequest().body("nonce not present");
    };
    let session_id: &str = session_id_header.to_str().unwrap();
    let Some(session_key) = session_manager::check_session(session_id) else {
        return HttpResponse::BadRequest().body("failed trying to get data");
    };
    let Some(data) = body.0.get("name") else {
        return HttpResponse::BadRequest().body("failed trying to get data");
    };
    let Ok(name_bytes) = BASE64.decode(data) else {
        return HttpResponse::BadRequest().body("failed decoding data");
    };
    let nonce = match BASE64.decode(nonce_header.to_str().unwrap()) {
        Ok(nonce) => nonce,
        Err(_) => {
            return HttpResponse::BadRequest().body("failed trying to get nonce");
        }
    };
    let Some(sesion_info) = session_manager::check_session(session_id) else {
        return HttpResponse::BadRequest().body("session not found");
    };
    let Ok(deciphered_data) = crypto_manager::decrypt_aes_gcm(
        &name_bytes,
        &nonce,
        &sesion_info.aes_key
    ) else {
        return HttpResponse::BadRequest().finish();
    };
    let Ok(ciphered_result) = crypto_manager::encrypt_aes_gcm(
        &deciphered_data,
        &sesion_info.aes_key
    ) else {
        return HttpResponse::BadRequest().finish();
    };
    let ciphered_data = HashMap::from([
        ("data".to_string(), crypto_manager::base64_encode(&ciphered_result.0)),
        ("nonce".to_string(), crypto_manager::base64_encode(&ciphered_result.1)),
    ]);
    HttpResponse::Ok().json(ciphered_data)
}
