use sea_orm::{ Database, DatabaseConnection };

pub async fn get_conn() -> DatabaseConnection {
    Database::connect("sqlite://db.sqlite").await.expect("error while connecting to database")
}
