use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserRegister {
    pub name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UserLogin {
    pub email: String,
    pub password: String,
}
