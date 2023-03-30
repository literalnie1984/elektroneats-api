use entity::{dinner, extras};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRegister {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserLogin {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserChangePassword {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Serialize)]
pub struct UserJson {
    pub username: String,
    pub admin: bool,
}

#[derive(Serialize)]
pub struct MenuOneDay {
    pub dinners: Vec<dinner::Model>,
    pub extras: Vec<extras::Model>,
}
