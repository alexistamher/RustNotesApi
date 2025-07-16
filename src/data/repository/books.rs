use sea_orm::{
    prelude::Uuid,
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait,
    EntityTrait,
    ModelTrait,
    QueryFilter,
};

use crate::{
    data::db::{
        entities::books::{ self, ActiveModel as BooksAm, Entity as Books },
        models::books::Books as PartialBooks,
        util::get_conn,
    },
    domain::models::request::books::BookAdd,
    util::result_util::MapErrorToString,
};

pub async fn get_all(user_id: &str) -> Result<Vec<PartialBooks>, String> {
    let conn = get_conn().await;
    Books::find()
        .filter(books::Column::UserId.eq(user_id))
        .into_partial_model::<PartialBooks>()
        .all(&conn).await
        .map_err(|e| e.to_string())
}

pub async fn add(book: BookAdd) -> Result<String, String> {
    let conn = get_conn().await;
    let id = book.id.unwrap_or(Uuid::new_v4().to_string());
    let active = BooksAm {
        id: Set(id),
        name: Set(book.name),
        user_id: Set(book.user_id),
        created: Set(book.created),
        last_modified: Set(book.last_modified),
    };
    active
        .insert(&conn).await
        .map(|n| n.id)
        .map_err(|e| e.to_string())
}

pub async fn remove(book_id: String) -> Result<(), String> {
    let conn = get_conn().await;
    let result = Books::find().filter(books::Column::Id.eq(book_id)).one(&conn).await;
    let Ok(model) = result else {
        return Err(result.err().unwrap().to_string());
    };
    let Some(book_found) = model else {
        return Err("book does not exist".to_string());
    };
    book_found
        .delete(&conn).await
        .map_err_as_str()
        .map(|_| ())
}
