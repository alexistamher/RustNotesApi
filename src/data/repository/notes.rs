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
        entities::notes::{ self, ActiveModel as NotesAm, Entity as Notes, Model },
        models::notes::Notes as PartialNotes,
        util::get_conn,
    },
    domain::models::request::notes::NoteAdd, util::result_util::MapErrorToString,
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
        .map_err_as_str()
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
    if note.id.is_none() {
        return insert_note(&note).await;
    }
    let current = Notes::find_by_id(note.id.clone().unwrap())
        .one(&conn).await
        .map_err_as_str()
        .map(|n| n.ok_or("note not found".to_string()))?;
    if current.is_ok() {
        let current: NotesAm = current.unwrap().into_active(&note);
        return current
            .update(&conn).await
            .map_err(|e| e.to_string())
            .map(|n| n.into());
    }
    insert_note(&note).await
}

async fn insert_note(note: &NoteAdd) -> Result<PartialNotes, String> {
    let conn = get_conn().await;
    let active: NotesAm = note.into();

    active
        .insert(&conn).await
        .map_err(|e| e.to_string())
        .map(|n| n.into())
}

pub async fn remove(note_id: String) -> Result<(), String> {
    let conn = get_conn().await;
    let result = Notes::find().filter(notes::Column::Id.eq(note_id)).one(&conn).await;
    let Ok(model) = result else {
        return Err(result.err().unwrap().to_string());
    };
    let Some(note_found) = model else {
        return Err("note does not exist".to_string());
    };
    note_found
        .delete(&conn).await
        .map_err_as_str()
        .map(|_| ())
}

impl From<Model> for PartialNotes {
    fn from(note: Model) -> Self {
        PartialNotes {
            id: note.id,
            title: note.title,
            content: note.content,
            user_id: note.user_id,
            book_id: note.book_id,
            created: note.created,
            last_modified: note.last_modified,
        }
    }
}

trait IntoActive {
    type Compl;
    type Res;
    fn into_active(self, note: &Self::Compl) -> Self::Res;
}

impl IntoActive for Model {
    type Compl = NoteAdd;
    type Res = NotesAm;

    fn into_active(self, note: &NoteAdd) -> NotesAm {
        let id = if note.id.is_some() {
            note.id.to_owned().unwrap()
        } else {
            Uuid::new_v4().to_string()
        };
        let created = Set(self.created);
        let last_modified = Set(chrono::Utc::now().timestamp_millis());
        let book_id = if note.book_id.is_some() {
            Set(note.book_id.clone().unwrap())
        } else {
            NotSet
        };
        NotesAm {
            id: Set(id),
            title: Set(note.title.to_owned()),
            content: Set(note.content.to_owned()),
            user_id: Set(note.user_id.to_owned()),
            book_id: book_id.into(),
            created,
            last_modified,
        }
    }
}

impl From<&NoteAdd> for NotesAm {
    fn from(note: &NoteAdd) -> Self {
        let id = if note.id.is_some() {
            note.id.to_owned().unwrap()
        } else {
            Uuid::new_v4().to_string()
        };
        let created = NotSet;
        let last_modified = Set(chrono::Utc::now().timestamp_millis());
        let book_id = if note.book_id.is_some() {
            Set(note.book_id.clone().unwrap())
        } else {
            NotSet
        };
        NotesAm {
            id: Set(id),
            title: Set(note.title.to_owned()),
            content: Set(note.content.to_owned()),
            user_id: Set(note.user_id.to_owned()),
            book_id: book_id.into(),
            created,
            last_modified,
        }
    }
}
