use sea_orm::{ DerivePartialModel };
use serde::Serialize;
use super::super::entities::users::Entity as UsersEntity;

#[derive(Debug, Serialize, PartialEq, DerivePartialModel)]
#[sea_orm(entity = "UsersEntity", from_query_result)]
pub struct Users {
    #[sea_orm(primary_key, auto_increment = false, column_type = "custom(\"UUID\")")]
    pub id: String,
    name: String,
    last_name: String,
    email: String,
    photo: Option<String>,
}
