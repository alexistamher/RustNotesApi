use actix_web::{ delete, get, post, web::{ self, Json, Path }, HttpRequest, HttpResponse };
use crate::{
    data::repository::books as books_service,
    domain::models::request::books::BookAdd,
    routes::util::ResultToHttp,
    util::token_manager::UserIdExtractor,
};

pub fn books_config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_books).service(add).service(remove);
}

#[get("books")]
async fn get_books(req: HttpRequest) -> HttpResponse {
    match req.user_id() {
        Ok(user_id) => books_service::get_all(&user_id).await.to_http(),
        Err(response) => response,
    }
}

#[post("books")]
async fn add(body: Json<BookAdd>) -> HttpResponse {
    books_service::add(body.into_inner()).await.to_http()
}

#[delete("books/{book_id}")]
async fn remove(query: Path<(String,)>) -> HttpResponse {
    let (book_id,) = query.into_inner();
    books_service::remove(book_id).await.to_http()
}
