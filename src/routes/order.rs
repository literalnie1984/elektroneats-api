use std::mem;

use actix_web::{post, web, get};
use entity::{dinner_orders, user_dinner_orders, extras_order, dinner, extras};
use log::{error, info};
use sea_orm::{Set, EntityTrait, QueryFilter, ColumnTrait, LoaderTrait};

use crate::{jwt_auth::AuthUser, errors::ServiceError, routes::structs::{OrderRequest, DinnerResponse}, appstate::AppState};

use super::structs::OrderResponse;

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
        error!("Database error creating dinner_orders: {}", e);
        ServiceError::InternalError
    })?;

    for dinner in order.dinners {
        let dinner_order_junction = user_dinner_orders::ActiveModel {
            order_id: Set(dinner_order_res.last_insert_id),
            dinner_id: Set(dinner.dinner_id),
            ..Default::default()
        };

        let dinner_order_res = user_dinner_orders::Entity::insert(dinner_order_junction).exec(db).await.map_err(|e| {
            error!("Database error creating user_dinner_orders: {}", e);
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
            error!("Database error creating extras_order: {}", e);
            ServiceError::InternalError
        })?;
    }

    Ok("Order created successfully".to_string())
}

#[get("/get-user-active-orders")]
async fn get_user_orders(/*user: AuthUser,*/ data: web::Data<AppState>) -> Result<web::Json<Vec<OrderResponse>>, ServiceError>{
    let db = &data.conn;
    let user_id = 1;

    let orders = dinner_orders::Entity::find()
        .filter(dinner_orders::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(|e| {
            error!("Database error getting user orders: {}", e);
            ServiceError::InternalError
        })?;

    let user_dinner_orders = orders.load_many(user_dinner_orders::Entity, db)
        .await.map_err(|e| {
            error!("Database error getting user orders: {}", e);
            ServiceError::InternalError
        })?;
    
    let mut response: Vec<OrderResponse> = Vec::new();
    for (order, user_dinner) in orders.into_iter().zip(user_dinner_orders.into_iter()) {

        let dinner = user_dinner.load_one(dinner::Entity, db)
            .await.map_err(|e| {
                error!("Database error getting user orders: {}", e);
                ServiceError::InternalError
            })?;

        let extras = user_dinner.load_many_to_many(extras::Entity, extras_order::Entity, db).await.map_err(|e| {
            error!("Database error getting user orders: {}", e);
            ServiceError::InternalError
        })?;

        let mut dinners_with_extras = dinner.into_iter().zip(extras.into_iter()).map(|(dinner, extras)| {
            DinnerResponse{dinner: dinner.unwrap(), extras: extras}
        }).collect::<Vec<_>>();

        response.push(
            OrderResponse {
                collection_date: order.collection_date,
                dinners: mem::take(&mut dinners_with_extras),
            }
        );
    }

    Ok(web::Json(response))
}
