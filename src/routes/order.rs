use std::mem;

use actix_web::{post, web, get};
use entity::{dinner_orders, user_dinner_orders, extras_order, dinner, extras, user};
use sea_orm::{Set, EntityTrait, QueryFilter, ColumnTrait, LoaderTrait, DatabaseConnection};

use crate::{jwt_auth::AuthUser, errors::ServiceError, routes::structs::{OrderRequest, DinnerResponse}, appstate::AppState, map_db_err};

use super::structs::{OrderResponse, UserWithOrders};

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

    let dinner_order_res = dinner_orders::Entity::insert(dinner_order).exec(db).await.map_err(map_db_err)?;

    for dinner in order.dinners {
        let dinner_order_junction = user_dinner_orders::ActiveModel {
            order_id: Set(dinner_order_res.last_insert_id),
            dinner_id: Set(dinner.dinner_id),
            ..Default::default()
        };

        let dinner_order_res = user_dinner_orders::Entity::insert(dinner_order_junction).exec(db).await.map_err(map_db_err)?;

        let vector = dinner.extras_ids.into_iter().map(|extra_id|{
            extras_order::ActiveModel {
                user_dinner_id: Set(dinner_order_res.last_insert_id),
                extras_id: Set(extra_id),
                ..Default::default()
            }
        }).collect::<Vec<_>>();
    
        extras_order::Entity::insert_many(vector).exec(db).await.map_err(map_db_err)?;
    }

    Ok("Order created successfully".to_string())
}

async fn get_user_orders(user_id: i32, db: &DatabaseConnection, realized: i8) -> Result<web::Json<Vec<OrderResponse>>, ServiceError>{
    let orders = dinner_orders::Entity::find()
        .filter(dinner_orders::Column::UserId.eq(user_id))
        .filter(dinner_orders::Column::Completed.eq(realized))
        .all(db)
        .await
        .map_err(map_db_err)?;

    let user_dinner_orders = orders.load_many(user_dinner_orders::Entity, db)
        .await.map_err(map_db_err)?;
    
    let mut response: Vec<OrderResponse> = Vec::new();
    for (order, user_dinner) in orders.into_iter().zip(user_dinner_orders.into_iter()) {

        let dinner = user_dinner.load_one(dinner::Entity, db)
            .await.map_err(map_db_err)?;

        let extras = user_dinner.load_many_to_many(extras::Entity, extras_order::Entity, db).await.map_err(map_db_err)?;

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

#[get("/completed-user-orders")]
async fn get_completed_user_orders(user: AuthUser, data: web::Data<AppState>) -> Result<web::Json<Vec<OrderResponse>>, ServiceError>{
    let db = &data.conn;
    let user_id = user.id;

    get_user_orders(user_id, db, 1).await
}

#[get("/pending-user-orders")]
async fn get_pending_user_orders(user: AuthUser, data: web::Data<AppState>) -> Result<web::Json<Vec<OrderResponse>>, ServiceError>{
    let db = &data.conn;
    let user_id = user.id;

    get_user_orders(user_id, db, 0).await
}

//TODO: ADD ADMIN RESTRICTION
//TODO: FIX RESPONSE REDUNDANCY
#[get("/all-pending-orders")]
async fn get_all_orders(data: web::Data<AppState>) -> Result<web::Json<Vec<UserWithOrders>>, ServiceError>{
    let db = &data.conn;

    let users_with_orders = user::Entity::find()
        .find_with_related(dinner_orders::Entity)
        .all(db)
        .await
        .map_err(map_db_err)?;

    let mut output: Vec<UserWithOrders> = Vec::with_capacity(users_with_orders.len());

    for (user, orders) in users_with_orders.iter(){
        output.push(
            UserWithOrders{
                username: user.username.clone(),
                user_id: user.id,
                orders: Vec::new(),
            }
        );

        let user_dinner_orders = orders.load_many(user_dinner_orders::Entity, db).await.map_err(map_db_err)?;

        for (user_dinner, order) in user_dinner_orders.iter().zip(orders.iter()) {

            let dinner = user_dinner.load_one(dinner::Entity, db).await.map_err(map_db_err)?;
            let extras = user_dinner.load_many_to_many(extras::Entity, extras_order::Entity, db).await.map_err(map_db_err)?;

            let mut dinners_with_extras = dinner.into_iter().zip(extras.into_iter()).map(|(dinner, extras)| {
                DinnerResponse{dinner: dinner.unwrap(), extras: extras}
            }).collect::<Vec<_>>();

            output.last_mut().unwrap().orders.push(
                OrderResponse {
                    collection_date: order.collection_date,
                    dinners: mem::take(&mut dinners_with_extras),
                }
            );
        }
    }


    Ok(web::Json(output))
}
