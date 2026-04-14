use std::io::Write;

use actix_multipart::Multipart;
use actix_web::web;
use chrono::Utc;
use futures::TryStreamExt;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, SelectColumns,
};
use uuid::Uuid;

use crate::{
    data::db::{
        entities::users::{self, ActiveModel as UsersAm, Entity as Users},
        models::users::Users as PartialUsers,
        util::get_conn,
    },
    domain::models::request::users::UserRegister,
    util::result_util::MapErrorToString,
};

pub async fn login(email: &str, password: &str) -> Result<String, String> {
    let conn = get_conn().await;
    Users::find()
        .select_column(users::Column::Id)
        .filter(
            users::Column::Email
                .eq(email)
                .and(users::Column::Password.eq(password)),
        )
        .into_tuple::<(String,)>()
        .one(&conn)
        .await
        .map_err(|e| {
            dbg!(&e);
            e.to_string()
        })
        .map(|u| u.ok_or("wrong credentials".to_string()).map(|u| u.0))?
}

pub async fn get_by_id(user_id: &str) -> Result<PartialUsers, String> {
    let conn = get_conn().await;
    Users::find()
        .filter(users::Column::Id.eq(user_id))
        .into_partial_model::<PartialUsers>()
        .one(&conn)
        .await
        .map_err(|e| e.to_string())
        .map(|e| e.ok_or("user info not found".to_string()))?
}

pub async fn register(user: UserRegister) -> Option<String> {
    let conn = get_conn().await;

    let active = UsersAm {
        id: Set(Uuid::new_v4().to_string()),
        name: Set(user.name),
        last_name: Set(user.last_name),
        email: Set(user.email),
        password: Set(user.password),
        ..Default::default()
    };

    active.insert(&conn).await.map(|n| n.id).ok()
}

pub async fn update_photo(user_id: &str, file_name: &str) -> Result<(), String> {
    let conn = get_conn().await;
    let mut active: UsersAm = Users::find()
        .filter(users::Column::Id.eq(user_id))
        .one(&conn)
        .await
        .map_err(|e| e.to_string())
        .map(|u| u.ok_or("user not found"))?
        .map(|u| u.into())?;

    active.photo = Set(Some(file_name.to_owned()));

    active.update(&conn).await.map_err_as_str().map(|_| ())
}

pub async fn delete_photo(user_id: &str) -> Result<(), String> {
    let conn = get_conn().await;
    let mut active: UsersAm = Users::find()
        .filter(users::Column::Id.eq(user_id))
        .one(&conn)
        .await
        .map_err(|e| e.to_string())
        .map(|u| u.ok_or("user not found"))?
        .map(|u| u.into())?;

    active.photo = Set(None);

    active.update(&conn).await.map_err_as_str().map(|_| ())
}

pub async fn save_photo(user_id: &str, mut payload: Multipart) -> Result<String, String> {
    while let Some(mut next) = payload.try_next().await.map_err_as_str()? {
        let Some(content_disposition) = next.content_disposition() else {
            return Err("file not provided".to_string());
        };
        let file_name = match content_disposition.get_filename() {
            Some(name) => name.to_owned(),
            None => Utc::now().timestamp().to_string(),
        };
        let file_ext = file_name.split('.').next_back().unwrap();

        let file_path: String = format!("./photos/{}.{}", &user_id, file_ext);
        let full_path = file_path.clone();
        let mut file = web::block(move || std::fs::File::create(file_path))
            .await
            .map_err_as_str()?
            .map_err_as_str()?;

        while let Some(chunk) = next.try_next().await.map_err_as_str()? {
            file = web::block(move || file.write_all(&chunk).map(|_| file))
                .await
                .map_err_as_str()?
                .map_err_as_str()?;
        }
        let _ = update_photo(user_id, &full_path).await;
    }
    Ok("Photo updated successfully".to_string())
}
