use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BookAdd {
    pub name: String,
    pub user_id: String,
}