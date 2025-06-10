use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{ NotSet, Set },
    ColumnTrait,
    EntityOrSelect,
    EntityTrait,
    ModelTrait,
    QueryFilter,
    QuerySelect,
    TryIntoModel,
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

pub async fn get_by_note_id(note_id: &str) -> Result<PartialNotes, String> {
    let conn = get_conn().await;
    Notes::find()
        .filter(notes::Column::Id.eq(note_id))
        .into_partial_model::<PartialNotes>()
        .one(&conn).await
        .map_err(|e| e.to_string())
        .map(|n| n.ok_or("note not found".to_string()))?
}

pub async fn get_all(user_id: &str) -> Result<Vec<PartialNotes>, String> {
    let conn = get_conn().await;
    Notes::find()
        .filter(notes::Column::UserId.eq(user_id))
        .into_partial_model::<PartialNotes>()
        .all(&conn).await
        .map_err(|e| e.to_string())
}

pub async fn add(note: NoteAdd) -> Result<PartialNotes, String> {
    let conn = get_conn().await;

    (
        if note.id.is_none() {
            let active = NotesAm {
                id: Set(Uuid::new_v4().to_string()),
                title: Set(note.title),
                content: Set(note.content),
                user_id: Set(note.user_id),
                book_id: NotSet, //Set(note.book_id),
                created: Set(chrono::Utc::now().timestamp_millis() as i32),
                last_modified: Set(chrono::Utc::now().timestamp_millis() as i32),
            };

            active
                .insert(&conn).await
                .map(|n| n.into())
                .map_err(|e| e.to_string())
        } else {
            let mut active: NotesAm = Notes::find_by_id(note.id.unwrap())
                .one(&conn).await
                .map_err(|e| e.to_string())?
                .unwrap()
                .into();
            if !&active.title.clone().unwrap().eq(&note.title) {
                active.title = Set(note.title);
            }
            if !&active.content.clone().unwrap().eq(&note.content) {
                active.content = Set(note.content);
            }
            if active.is_changed() {
                active.last_modified = Set(chrono::Utc::now().timestamp_millis() as i32);
            }

            active.save(&conn).await.map_err(|e| e.to_string())
        }
    )
        .map(|a| a.try_into_model().ok().ok_or("failed getting model".to_string()).unwrap())
        .map(|n| PartialNotes {
            id: n.id,
            title: n.title,
            content: n.content,
            user_id: n.user_id,
            book_id: n.book_id,
            created: n.created,
            last_modified: n.last_modified,
        })
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
