use entity::sea_orm_active_enums::Type;
use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
pub struct UserRegister {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
pub struct UserLogin {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
pub struct UserChangePassword {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Apiv2Schema, Serialize)]
pub struct Dinner {
    pub id: i32,
    pub name: String,
    pub price: f32,
    pub image: String,
    pub week_day: u8,
    pub max_supply: i32,
    pub r#type: DinnerType,
}

#[derive(Apiv2Schema, Serialize)]
pub struct Extras {
    pub id: i32,
    pub name: String,
    pub price: f32,
}

#[derive(Apiv2Schema, Serialize)]
pub enum DinnerType {
    Soup,
    Main,
}

impl From<Type> for DinnerType {
    fn from(value: Type) -> Self {
        match value {
            Type::Soup => Self::Soup,
            Type::Main => Self::Main,
        }
    }
}
