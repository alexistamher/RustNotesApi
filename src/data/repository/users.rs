use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait,
    EntityTrait,
    QueryFilter,
    SelectColumns,
};
use uuid::Uuid;

use crate::{
    data::db::{
        entities::users::{ self, ActiveModel as UsersAm, Entity as Users },
        models::users::Users as PartialUsers,
        util::get_conn,
    },
    domain::models::request::{ users::UserRegister },
};

pub async fn login(email: &str, password: &str) -> Result<String, String> {
    let conn = get_conn().await;
    Users::find()
        .select_column(users::Column::Id)
        .filter(users::Column::Email.eq(email).and(users::Column::Password.eq(password)))
        .into_tuple::<(String,)>()
        .one(&conn).await
        .map(|u| u.expect("user not found").0)
        .map_err(|e| e.to_string())
}

pub async fn get_by_id(user_id: &str) -> Result<PartialUsers, String> {
    let conn = get_conn().await;
    Users::find()
        .filter(users::Column::Id.eq(user_id))
        .into_partial_model::<PartialUsers>()
        .one(&conn).await
        .map(|e| { e.expect("user not found") })
        .map_err(|e| e.to_string())
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

    active
        .insert(&conn).await
        .map(|n| n.id)
        .ok()
}
