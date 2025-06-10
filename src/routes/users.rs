use actix_web::{ get, post, web::{ self, Json }, HttpRequest, HttpResponse };
use crate::{
    data::repository::users::{ self as users_service, get_by_id },
    domain::models::request::users::{ UserLogin, UserRegister },
    routes::util::ResultToHttp,
    util::token_manager::{ check_token, generate_token, revoke_creds, TokenExtractor },
};

pub fn users_config(cfg: &mut web::ServiceConfig) {
    cfg.service(info).service(login).service(logout).service(register).service(refresh);
}

#[get("user/info")]
async fn info(req: HttpRequest) -> HttpResponse {
    let token = match req.token() {
        Ok(token) => token,
        Err(response) => {
            return response;
        }
    };
    let Some(claims) = check_token(&token) else {
        return HttpResponse::Unauthorized().finish();
    };
    get_by_id(&claims.user_id).await.to_http()
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
    let token = match req.token() {
        Ok(token) => token,
        Err(response) => {
            return response;
        }
    };
    let Some(claims) = check_token(&token) else {
        return HttpResponse::Unauthorized().finish();
    };
    let token = generate_token(&claims.user_id);
    HttpResponse::Ok().json(token)
}

#[post("user/logout")]
async fn logout(req: HttpRequest) -> HttpResponse {
    let token = match req.token() {
        Ok(token) => token,
        Err(response) => {
            return response;
        }
    };
    let Some(claims) = check_token(&token) else {
        return HttpResponse::Unauthorized().finish();
    };
    revoke_creds(&claims.jti);
    HttpResponse::Ok().finish()
}

#[post("user/register")]
async fn register(body: Json<UserRegister>) -> HttpResponse {
    users_service::register(body.into_inner()).await.to_http()
}
