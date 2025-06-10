use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{ NotSet, Set },
    ColumnTrait,
    EntityTrait,
    ModelTrait,
    QueryFilter,
};
use uuid::Uuid;

use crate::{
    data::db::{
        entities::notes::{ self, ActiveModel as NotesAm, Entity as Notes },
        models::notes::Notes as PartialNotes,
        util::get_conn,
    },
    domain::models::request::notes::NoteAdd,
};

pub async fn get_by_book_id(book_id: &str) -> Result<Vec<PartialNotes>, String> {
    let conn = get_conn().await;
    Notes::find()
        .filter(notes::Column::BookId.eq(book_id))
        .into_partial_model::<PartialNotes>()
        .all(&conn).await
        .map_err(|e| e.to_string())
}

pub async fn get_all() -> Result<Vec<PartialNotes>, String> {
    let conn = get_conn().await;
    Notes::find()
        .into_partial_model::<PartialNotes>()
        .all(&conn).await
        .map_err(|e| e.to_string())
}

pub async fn add(note: NoteAdd) -> Result<String, String> {
    let conn = get_conn().await;

    let active = NotesAm {
        id: Set(Uuid::new_v4().to_string()),
        title: Set(note.title),
        content: Set(note.content),
        user_id: Set(note.user_id),
        book_id: Set(note.book_id),
        created: NotSet,
        last_modified: Set(chrono::Utc::now().timestamp_subsec_millis() as i32),
    };

    active
        .save(&conn).await
        .map(|n| n.id.into_value().unwrap().to_string())
        .map_err(|e| e.to_string())
}

pub async fn remove(note_id: String) -> Result<(), String> {
    let conn = get_conn().await;
    // !! shorter way
    // Notes::delete_by_id(note_id)
    //     .exec(&conn).await
    //     .map(|_| ())
    //     .map_err(|e| e.to_string());
    let result = Notes::find().filter(notes::Column::Id.eq(note_id)).one(&conn).await;
    let Ok(model) = result else {
        return Err(result.err().unwrap().to_string());
    };
    let Some(note_found) = model else {
        return Err("note does not exist".to_string());
    };
    note_found
        .delete(&conn).await
        .map_err(|e| e.to_string())
        .map(|_| ())
}
