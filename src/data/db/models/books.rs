use sea_orm::{ DerivePartialModel };
use serde::Serialize;
use super::super::entities::books::Entity as BooksEntity;

#[derive(Debug, Serialize, PartialEq, DerivePartialModel)]
#[sea_orm(entity = "BooksEntity", from_query_result)]
pub struct Books {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub user_id: String,
    pub created: i64,
    pub last_modified: i64
}
