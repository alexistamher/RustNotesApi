use actix_web::HttpResponse;
use serde::Serialize;

pub trait ResultToHttp<T>
where
    T: Serialize,
{
    fn to_http(self) -> HttpResponse;
}

impl<T: Serialize> ResultToHttp<T> for Result<T, String> {
    fn to_http(self) -> HttpResponse {
        match self {
            Ok(body) => HttpResponse::Ok().json(body),
            Err(err) => HttpResponse::Forbidden().body(err),
        }
    }
}

impl<T: Serialize> ResultToHttp<T> for Option<T> {
    fn to_http(self) -> HttpResponse {
        match self {
            Some(body) => HttpResponse::Ok().json(body),
            None => HttpResponse::BadRequest().finish(),
        }
    }
}
