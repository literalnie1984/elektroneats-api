use chrono::{DateTime, Utc};
use entity::{dinner, extras};
use serde::{Serialize, Deserialize};
use chrono::serde::ts_seconds;

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dinner{
    pub dinner_id: i32,
    pub extras_ids: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRequest {
    pub dinners: Vec<Dinner>,
    #[serde(with = "ts_seconds")]
    pub collection_date: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DinnerResponse{
    pub dinner: dinner::Model,
    pub extras: Vec<extras::Model>,
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    #[serde(with = "ts_seconds")]
    pub collection_date: DateTime<Utc>,
    pub dinners: Vec<DinnerResponse>,
}