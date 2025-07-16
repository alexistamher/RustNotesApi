use actix_web::{ delete, get, post, web::{ self, Json, Path }, HttpRequest, HttpResponse };
use crate::{
    data::repository::notes as notes_service,
    domain::models::request::notes::NoteAdd,
    routes::util::ResultToHttp,
    util::token_manager::UserIdExtractor,
};

pub fn notes_config(cfg: &mut web::ServiceConfig) {
    cfg.service(all).service(get_by_book_id).service(add).service(remove).service(get_by_note_id);
}

#[get("notes")]
async fn all(req: HttpRequest) -> HttpResponse {
    match req.user_id() {
        Ok(user_id) => notes_service::get_all(&user_id).await.to_http(),
        Err(response) => response,
    }
}

#[get("notes/book/{book_id}")]
async fn get_by_book_id(path: Path<(String,)>) -> HttpResponse {
    let (book_id,) = path.into_inner();
    notes_service::get_by_book_id(&book_id).await.to_http()
}

#[get("notes/{note_id}")]
async fn get_by_note_id(path: Path<(String,)>) -> HttpResponse {
    let (note_id,) = path.into_inner();
    notes_service::get_by_note_id(&note_id).await.to_http()
}

#[post("notes")]
async fn add(body: Json<NoteAdd>) -> HttpResponse {
    notes_service::add(body.into_inner()).await.to_http()
}

#[delete("notes/{note_id}")]
async fn remove(query: Path<(String,)>) -> HttpResponse {
    let (note_id,) = query.into_inner();
    notes_service::remove(note_id).await.to_http()
}
