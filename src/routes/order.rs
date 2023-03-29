use actix_web::{post, web};
use chrono::{NaiveDate, FixedOffset, DateTime, TimeZone};
use entity::{dinner_orders, dinner, user_dinner_orders, extras_order};
use log::{error, info};
use sea_orm::{Set, EntityTrait};

use crate::{jwt_auth::AuthUser, errors::ServiceError, routes::structs::OrderRequest, appstate::AppState};

#[post("/create")]
async fn create_order(user: AuthUser, data: web::Data<AppState>, order: web::Json<OrderRequest>) -> Result<String, ServiceError>{
    let db = &data.conn;
    let order = order.into_inner();
    let user_id = user.id;

    // let tz_offset = FixedOffset::east_opt(2 * 3600).unwrap();
    // let dt_with_tz: DateTime<FixedOffset> = tz_offset.from_utc_datetime(&order.collection_date.naive_utc());

    let dinner_order = dinner_orders::ActiveModel {
        user_id: Set(user_id),
        collection_date: Set(order.collection_date),
        ..Default::default()
    };

    let dinner_order_res = dinner_orders::Entity::insert(dinner_order).exec(db).await.map_err(|e| {
        error!("Database error creating order: {}", e);
        ServiceError::InternalError
    })?;

    let dinner_order_junction = user_dinner_orders::ActiveModel {
        order_id: Set(dinner_order_res.last_insert_id),
        dinner_id: Set(order.dinner_id),
        ..Default::default()
    };

    let dinner_order_res = user_dinner_orders::Entity::insert(dinner_order_junction).exec(db).await.map_err(|e| {
        error!("Database error creating order: {}", e);
        ServiceError::InternalError
    })?;

    info!("Order created with id: {}", dinner_order_res.last_insert_id);

    let vector = order.extras_ids.into_iter().map(|extra_id|{
        extras_order::ActiveModel {
            user_dinner_id: Set(dinner_order_res.last_insert_id),
            extras_id: Set(extra_id),
            ..Default::default()
        }
    }).collect::<Vec<_>>();

    info!("Extras: {:?}", vector);

    let extras_order_res = extras_order::Entity::insert_many(vector).exec(db).await.map_err(|e| {
        error!("Database error creating order: {}", e);
        ServiceError::InternalError
    })?;

    Ok(format!("Order created with id: {}", extras_order_res.last_insert_id))
}