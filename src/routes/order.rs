use actix_web::{post, web};
use chrono::{NaiveDate, FixedOffset, DateTime, TimeZone};
use entity::{dinner_orders, dinner};
use log::error;
use sea_orm::{Set, EntityTrait};

use crate::{jwt_auth::AuthUser, errors::ServiceError, routes::structs::OrderRequest, appstate::AppState};

#[post("/create")]
async fn create_order(user: AuthUser, data: web::Data<AppState>, order: web::Json<OrderRequest>) -> Result<String, ServiceError>{
    let db = &data.conn;
    let order = order.into_inner();
    let user_id = user.id;

    let tz_offset = FixedOffset::east_opt(2 * 3600).unwrap();
    let dt_with_tz: DateTime<FixedOffset> = tz_offset.from_utc_datetime(&order.collection_date.naive_utc());

    Ok(format!("Cool date: {}", dt_with_tz))

    // let dinner_orders = dinner_orders::ActiveModel {
    //     user_id: Set(user_id),
    //     collection_date: Set(chrono::),
    //     ..Default::default()
    // };

    // let result = dinner_orders::Entity::insert(dinner_orders).exec(db).await.map_err(|e| {
    //     error!("Database error creating order: {}", e);
    //     ServiceError::InternalError
    // })?;

}