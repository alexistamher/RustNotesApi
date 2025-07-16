use sea_orm::{ DerivePartialModel };
use serde::Serialize;
use super::super::entities::notes::Entity as NotesEntity;

#[derive(Debug, Serialize, PartialEq, DerivePartialModel)]
#[sea_orm(entity = "NotesEntity", from_query_result)]
pub struct Notes {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub title: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    pub user_id: String,
    pub book_id: Option<String>,
    pub created: i64,
    pub last_modified: i64,
}
