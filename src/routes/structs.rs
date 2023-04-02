use std::collections::HashSet;

use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use entity::model_enums::Status;
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
#[serde(rename_all = "camelCase")]
pub struct TokenGenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct MenuResult3D {
    pub response: Vec<DinnerWithExtras>,
    pub extras: HashSet<extras::Model>,
}

#[derive(Serialize)]
pub struct MenuOneDay {
    pub dinners: Vec<dinner::Model>,
    pub extras: Vec<extras::Model>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DinnerWithExtras {
    pub dinners: Vec<dinner::Model>,
    pub extras_ids: Vec<i32>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct UpdateMenu {
    pub id: i32,
    pub name: Option<String>,
    pub price: Option<f32>,
    pub image: Option<String>,
    pub max_supply: Option<i32>,
    pub week_day: Option<entity::model_enums::Weekday>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dinner {
    pub dinner_id: i32,
    pub extras_ids: Vec<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LastUpdateResponse{
    #[serde(with = "ts_seconds")]
    pub last_update: DateTime<Utc>
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderStatusRequest {
    pub new_status: Status,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRequest {
    pub dinners: Vec<Dinner>,
    #[serde(with = "ts_seconds")]
    pub collection_date: DateTime<Utc>,
}
//DinnerResponse
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DinnerResponse {
    pub dinner_id: i32,
    pub extras_ids: Vec<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    pub order_id: i32,
    #[serde(with = "ts_seconds")]
    pub collection_date: DateTime<Utc>,
    pub status: Status,
    pub dinners: Vec<DinnerResponse>,
}

#[derive(Serialize)]
pub struct UserWithOrders {
    pub user_id: i32,
    pub username: String,
    pub orders: Vec<OrderResponse>,
}

#[derive(Deserialize)]
pub struct StripeUser {
    pub address: Address,
    pub name: String,
    pub phone: String,
}

#[derive(Deserialize)]
pub struct Address {
    pub city: String,
    pub country: String,
    pub postal_code: String,
    pub state: String,
}

#[derive(Serialize)]
pub struct AddReturn {
    pub customer_id: String,
    pub intent_secret: String,
}

#[derive(Serialize)]
pub struct AllUsersOrders {
    pub response: Vec<UserWithOrders>,
    pub dinners: HashSet<dinner::Model>,
    pub extras: HashSet<extras::Model>,
}

#[derive(Serialize)]
pub struct UserOrders {
    pub response: Vec<OrderResponse>,
    pub dinners: HashSet<dinner::Model>,
    pub extras: HashSet<extras::Model>,
}
