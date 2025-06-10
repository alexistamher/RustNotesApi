use actix_web::{ get, post, web::{ self, Json, Path }, HttpResponse };
use crate::{
    data::repository::notes as notes_service,
    domain::models::request::notes::NoteAdd,
    routes::util::ResultToHttp,
};

pub fn notes_config(cfg: &mut web::ServiceConfig) {
    cfg.service(all).service(get_by_book_id).service(add).service(remove);
}

#[get("notes/all")]
async fn all() -> HttpResponse {
    notes_service::get_all().await.to_http()
}

#[get("notes/{book_id}")]
async fn get_by_book_id(path: Path<(String,)>) -> HttpResponse {
    let (book_id,) = path.into_inner();
    notes_service::get_by_book_id(&book_id).await.to_http()
}

#[post("notes/add")]
async fn add(body: Json<NoteAdd>) -> HttpResponse {
    notes_service::add(body.into_inner()).await.to_http()
}

#[get("notes/remove/{note_id}")]
async fn remove(query: Path<(String,)>) -> HttpResponse {
    let (note_id,) = query.into_inner();
    notes_service::remove(note_id).await.to_http()
}
