use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BookAdd {
    pub id: Option<String>,
    pub name: String,
    pub user_id: String,
    pub created: i64,
    pub last_modified: i64,
}