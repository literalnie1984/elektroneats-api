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

    let dinner_order = dinner_orders::ActiveModel {
        user_id: Set(user_id),
        collection_date: Set(order.collection_date),
        ..Default::default()
    };

    let dinner_order_res = dinner_orders::Entity::insert(dinner_order).exec(db).await.map_err(|e| {
        error!("Database error creating order: {}", e);
        ServiceError::InternalError
    })?;

    for dinner in order.dinners {
        let dinner_order_junction = user_dinner_orders::ActiveModel {
            order_id: Set(dinner_order_res.last_insert_id),
            dinner_id: Set(dinner.dinner_id),
            ..Default::default()
        };

        let dinner_order_res = user_dinner_orders::Entity::insert(dinner_order_junction).exec(db).await.map_err(|e| {
            error!("Database error creating order: {}", e);
            ServiceError::InternalError
        })?;

        let vector = dinner.extras_ids.into_iter().map(|extra_id|{
            extras_order::ActiveModel {
                user_dinner_id: Set(dinner_order_res.last_insert_id),
                extras_id: Set(extra_id),
                ..Default::default()
            }
        }).collect::<Vec<_>>();
    
        extras_order::Entity::insert_many(vector).exec(db).await.map_err(|e| {
            error!("Database error creating order: {}", e);
            ServiceError::InternalError
        })?;
    }

    Ok(format!("Soj soj soj"))
}