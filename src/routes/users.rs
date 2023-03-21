use actix_web::{post, web, HttpResponse, Responder};
use entity::prelude::{Order, User};
use entity::{order, user};
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait, ModelTrait};

use crate::AppState;

#[post("/get-orders/{user_id}")]
async fn get_all_orders_for_user(
    user_id: web::Path<i32>,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    let conn = &data.conn;

    let user = User::find_by_id(user_id).one(conn).await.unwrap().unwrap();
    let orders = user.find_related(Order).all(conn).await.unwrap();

    HttpResponse::Ok().json(orders)
}

#[post("/add-order")]
async fn add_order(order: web::Json<order::Model>, data: web::Data<AppState>) -> impl Responder {
    let conn = &data.conn;

    let order = order.into_inner();

    order::ActiveModel {
        user_id: sea_orm::ActiveValue::set(order.user_id),
        product_id: ActiveValue::set(order.product_id),
        ..Default::default()
    }
    .save(conn)
    .await
    .unwrap();

    HttpResponse::Ok().body("Hello world")
}

#[post("/add")]
async fn add_user(user: web::Json<user::Model>, data: web::Data<AppState>) -> impl Responder {
    let conn = &data.conn;

    let user = user.into_inner();

    user::ActiveModel {
        username: ActiveValue::set(user.username),
        password: ActiveValue::set(user.password),
        ..Default::default()
    }
    .save(conn)
    .await
    .unwrap();

    HttpResponse::Ok().body("Hello world")
}
