use std::mem;

use actix_web::{get, post, web};
use entity::{dinner, dinner_orders, extras, extras_order, user_dinner_orders};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, LoaderTrait, QueryFilter, Set};

use crate::{
    appstate::AppState,
    convert_err_to_500,
    errors::ServiceError,
    jwt_auth::AuthUser,
    routes::structs::{DinnerResponse, OrderRequest},
};

use super::structs::OrderResponse;

#[post("/create")]
async fn create_order(
    user: AuthUser,
    data: web::Data<AppState>,
    order: web::Json<OrderRequest>,
) -> Result<String, ServiceError> {
    let db = &data.conn;
    let order = order.into_inner();
    let user_id = user.id;

    let dinner_order = dinner_orders::ActiveModel {
        user_id: Set(user_id),
        collection_date: Set(order.collection_date),
        ..Default::default()
    };

    let dinner_order_res = dinner_orders::Entity::insert(dinner_order)
        .exec(db)
        .await
        .map_err(|e| convert_err_to_500(e, Some("Database error creating dinner_orders: {}")))?;

    for dinner in order.dinners {
        let dinner_order_junction = user_dinner_orders::ActiveModel {
            order_id: Set(dinner_order_res.last_insert_id),
            dinner_id: Set(dinner.dinner_id),
            ..Default::default()
        };

        let dinner_order_res = user_dinner_orders::Entity::insert(dinner_order_junction)
            .exec(db)
            .await
            .map_err(|e| {
                convert_err_to_500(e, Some("Database error creating user_dinner_orders: {}"))
            })?;

        let vector = dinner
            .extras_ids
            .into_iter()
            .map(|extra_id| extras_order::ActiveModel {
                user_dinner_id: Set(dinner_order_res.last_insert_id),
                extras_id: Set(extra_id),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        extras_order::Entity::insert_many(vector)
            .exec(db)
            .await
            .map_err(|e| convert_err_to_500(e, Some("Database error creating extras_order: {}")))?;
    }

    Ok("Order created successfully".to_string())
}

async fn get_user_orders(
    user_id: i32,
    db: &DatabaseConnection,
    realized: i8,
) -> Result<web::Json<Vec<OrderResponse>>, ServiceError> {
    let db_err = |err| convert_err_to_500(err, Some("Database error getting user orders"));
    let orders = dinner_orders::Entity::find()
        .filter(dinner_orders::Column::UserId.eq(user_id))
        .filter(dinner_orders::Column::Completed.eq(realized))
        .all(db)
        .await
        .map_err(db_err)?;

    let user_dinner_orders = orders
        .load_many(user_dinner_orders::Entity, db)
        .await
        .map_err(db_err)?;

    let mut response: Vec<OrderResponse> = Vec::new();
    for (order, user_dinner) in orders.into_iter().zip(user_dinner_orders.into_iter()) {
        let dinner = user_dinner
            .load_one(dinner::Entity, db)
            .await
            .map_err(db_err)?;

        let extras = user_dinner
            .load_many_to_many(extras::Entity, extras_order::Entity, db)
            .await
            .map_err(db_err)?;

        let mut dinners_with_extras = dinner
            .into_iter()
            .zip(extras.into_iter())
            .map(|(dinner, extras)| DinnerResponse {
                dinner: dinner.unwrap(),
                extras,
            })
            .collect::<Vec<_>>();

        response.push(OrderResponse {
            collection_date: order.collection_date,
            dinners: mem::take(&mut dinners_with_extras),
        });
    }

    Ok(web::Json(response))
}

#[get("/completed-user-orders")]
async fn get_completed_user_orders(
    user: AuthUser,
    data: web::Data<AppState>,
) -> Result<web::Json<Vec<OrderResponse>>, ServiceError> {
    let db = &data.conn;
    let user_id = user.id;

    get_user_orders(user_id, db, 1).await
}

#[get("/pending-user-orders")]
async fn get_pending_user_orders(
    user: AuthUser,
    data: web::Data<AppState>,
) -> Result<web::Json<Vec<OrderResponse>>, ServiceError> {
    let db = &data.conn;
    let user_id = user.id;

    get_user_orders(user_id, db, 0).await
}
