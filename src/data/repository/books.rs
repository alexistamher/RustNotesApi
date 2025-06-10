use sea_orm::{
    prelude::Uuid, ActiveModelTrait, ActiveValue::{NotSet, Set}, ColumnTrait, EntityTrait, ModelTrait, QueryFilter
};

use crate::{
    data::db::{
        entities::books::{ self, ActiveModel as BooksAm, Entity as Books },
        models::books::Books as PartialBooks,
        util::get_conn,
    },
    domain::models::request::{ books::BookAdd },
};

pub async fn get_by_user_id(user_id: &str) -> Result<Vec<PartialBooks>, String> {
    let conn = get_conn().await;
    Books::find()
        .filter(books::Column::UserId.eq(user_id))
        .into_partial_model::<PartialBooks>()
        .all(&conn).await
        .map_err(|e| e.to_string())
}

pub async fn get_all() -> Result<Vec<PartialBooks>, String> {
    let conn = get_conn().await;
    Books::find()
        .into_partial_model::<PartialBooks>()
        .all(&conn).await
        .map_err(|e| e.to_string())
}

pub async fn add(book: BookAdd) -> Result<String, String> {
    let conn = get_conn().await;

    let active = BooksAm {
        id: Set(Uuid::new_v4().to_string()),
        name: Set(book.name),
        user_id: Set(book.user_id),
        created: NotSet,
        last_modified: Set(chrono::Utc::now().timestamp_subsec_millis() as i32),
    };

    active
        .save(&conn).await
        .map(|n| n.id.into_value().unwrap().to_string())
        .map_err(|e| e.to_string())
}

pub async fn remove(book_id: String) -> Result<(), String> {
    let conn = get_conn().await;
    // !! shorter way
    // Books::delete_by_id(book_id)
    //     .exec(&conn).await
    //     .map(|_| ())
    //     .map_err(|e| e.to_string());
    let result = Books::find().filter(books::Column::Id.eq(book_id)).one(&conn).await;
    let Ok(model) = result else {
        return Err(result.err().unwrap().to_string());
    };
    let Some(book_found) = model else {
        return Err("book does not exist".to_string());
    };
    book_found
        .delete(&conn).await
        .map_err(|e| e.to_string())
        .map(|_| ())
}
