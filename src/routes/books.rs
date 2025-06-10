use actix_web::{ get, post, web::{ self, Json, Path }, HttpResponse };
use crate::{
    data::repository::books as books_service,
    domain::models::request::books::BookAdd,
    routes::util::ResultToHttp,
};

pub fn books_config(cfg: &mut web::ServiceConfig) {
    cfg.service(all).service(get_by_book_id).service(add).service(remove);
}

#[get("books/all")]
async fn all() -> HttpResponse {
    books_service::get_all().await.to_http()
}

#[get("books/{user_id}")]
async fn get_by_book_id(path: Path<(String,)>) -> HttpResponse {
    let (user_id,) = path.into_inner();
    books_service::get_by_user_id(&user_id).await.to_http()
}

#[post("books/add")]
async fn add(body: Json<BookAdd>) -> HttpResponse {
    books_service::add(body.into_inner()).await.to_http()
}

#[get("books/remove/{book_id}")]
async fn remove(query: Path<(String,)>) -> HttpResponse {
    let (book_id,) = query.into_inner();
    books_service::remove(book_id).await.to_http()
}
