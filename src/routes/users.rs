use std::fs;

use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{ delete, get, post, web::{ self, Json }, HttpRequest, HttpResponse };
use crate::{
    data::repository::users::{ self as users_service, get_by_id },
    domain::models::request::users::{ UserLogin, UserRegister },
    routes::util::ResultToHttp,
    util::{
        cache_manager::revoke_token,
        token_manager::{ generate_token, revoke_creds, ClaimsExtractor, UserIdExtractor },
    },
};

pub fn users_config(cfg: &mut web::ServiceConfig) {
    cfg.service(info)
        .service(login)
        .service(logout)
        .service(register)
        .service(refresh)
        .service(upload_photo)
        .service(delete_photo)
        .service(user_photo);
}

#[get("user/info")]
async fn info(req: HttpRequest) -> HttpResponse {
    match req.user_id() {
        Ok(user_id) => get_by_id(&user_id).await.to_http(),
        Err(response) => response,
    }
}

#[post("user/login")]
async fn login(body: Json<UserLogin>) -> HttpResponse {
    let UserLogin { email, password } = body.into_inner();
    match users_service::login(&email, &password).await {
        Ok(user_id) => {
            let token = generate_token(&user_id);
            HttpResponse::Ok().json(token)
        }
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

#[get("user/refresh")]
async fn refresh(req: HttpRequest) -> HttpResponse {
    let claims = match req.claims() {
        Ok(claims) => claims,
        Err(response) => {
            return response;
        }
    };
    revoke_token(&claims.jti, claims.exp as i64);
    let token = generate_token(&claims.user_id);
    HttpResponse::Ok().json(token)
}

#[get("user/logout")]
async fn logout(req: HttpRequest) -> HttpResponse {
    let claims = match req.claims() {
        Ok(claims) => claims,
        Err(response) => {
            return response;
        }
    };
    let lifetime = (chrono::Utc::now() - chrono::Duration::seconds(claims.exp as i64)).timestamp();
    if lifetime > 0 {
        return HttpResponse::Ok().finish();
    }
    revoke_creds(&claims.jti, claims.exp as i64);
    HttpResponse::Ok().finish()
}

#[post("user/register")]
async fn register(body: Json<UserRegister>) -> HttpResponse {
    let response = users_service::register(body.into_inner()).await;
    if let Some(user_id) = response {
        let token = generate_token(&user_id);
        return HttpResponse::Ok().json(token);
    }
    HttpResponse::Forbidden().finish()
}

#[post("user/photo")]
pub async fn upload_photo(payload: Multipart, req: HttpRequest) -> HttpResponse {
    let user_id = match req.user_id() {
        Ok(data) => data,
        Err(response) => {
            return response;
        }
    };
    users_service::save_photo(&user_id, payload).await.to_http()
}

#[delete("user/photo")]
async fn delete_photo(req: HttpRequest) -> HttpResponse {
    let claims = match req.claims() {
        Ok(claims) => claims,
        Err(response) => {
            return response;
        }
    };
    let mut entries = fs::read_dir("./photos/").unwrap();
    let file_found = entries.find(|f| {
        let path_b = f.as_ref().ok().unwrap().path();
        if path_b.is_file() && path_b.to_string_lossy().contains(&claims.user_id) {
            return true;
        }
        false
    });
    let Some(file) = file_found else {
        return HttpResponse::Forbidden().body("file not found");
    };
    let Ok(file_dir) = file else {
        return HttpResponse::Forbidden().body("something went wrong");
    };
    let path = &file_dir.path().to_string_lossy().into_owned();
    match users_service::delete_photo(&claims.user_id).await {
        Ok(_) => (),
        Err(err) => {
            return HttpResponse::BadRequest().body(err);
        }
    }
    match fs::remove_file(path) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => { HttpResponse::BadRequest().body(e.to_string()) }
    }
}

#[get("user/photo")]
async fn user_photo(req: HttpRequest) -> HttpResponse {
    let user_id = match req.clone().user_id() {
        Ok(data) => data,
        Err(response) => {
            return response;
        }
    };
    let mut entries = fs::read_dir("./photos/").unwrap();
    let file_found = entries.find(|f| {
        let path_b = f.as_ref().ok().unwrap().path();
        if path_b.is_file() && path_b.to_string_lossy().contains(&user_id) {
            return true;
        }
        false
    });
    let Some(file) = file_found else {
        return HttpResponse::Forbidden().body("file not found");
    };
    let Ok(file_dir) = file else {
        return HttpResponse::Forbidden().body("something went wrong");
    };
    let path = file_dir.file_name().into_string().unwrap();
    let Ok(file) = NamedFile::open(format!("./photos/{}", path)) else {
        return HttpResponse::Forbidden().body("unable to get file");
    };
    file.into_response(&req)
}
