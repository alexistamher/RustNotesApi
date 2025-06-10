use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NoteAdd {
    pub title: Option<String>,
    pub content: String,
    pub user_id: String,
    pub book_id: Option<String>,
}
