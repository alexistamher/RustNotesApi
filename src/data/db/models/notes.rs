use sea_orm::{ DerivePartialModel };
use serde::Serialize;
use super::super::entities::notes::Entity as NotesEntity;

#[derive(Debug, Serialize, PartialEq, DerivePartialModel)]
#[sea_orm(entity = "NotesEntity", from_query_result)]
pub struct Notes {
    #[sea_orm(primary_key, auto_increment = false, column_type = "custom(\"UUID\")")]
    pub id: String,
    title: Option<String>,
    #[sea_orm(column_type = "Text")]
    content: String,
    #[sea_orm(column_type = "custom(\"UUID\")")]
    user_id: String,
    #[sea_orm(column_type = "custom(\"UUID\")")]
    book_id: Option<String>,
}
