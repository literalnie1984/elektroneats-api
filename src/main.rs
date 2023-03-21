use actix_web::{App, HttpResponse, HttpServer, Responder, web, post};
use migration::{Migrator, MigratorTrait, DbErr};
use sea_orm::{EntityTrait,DatabaseConnection, ModelTrait, ActiveValue, ActiveModelTrait};

use entity::{user, order};
use entity::prelude::{User, Order};

#[derive(Debug, Clone)]
struct AppState {
    conn: DatabaseConnection,
}

#[post("/get-orders/{user_id}")]
async fn get_all_orders_for_user(user_id: web::Path<i32>, data: web::Data<AppState>) -> impl Responder {
    let user_id = user_id.into_inner();
    let conn = &data.conn;

    let user = User::find_by_id(user_id).one(conn).await.unwrap().unwrap();
    let orders = user.find_related(Order)
        .all(conn)
        .await
        .unwrap();

    HttpResponse::Ok().json(orders)
}

#[post("/add-order")]
async fn add_order(order: web::Json<order::Model>, data: web::Data<AppState>) -> impl Responder {
    let conn = &data.conn;

    let order = order.into_inner();

    order::ActiveModel {
        user_id: ActiveValue::set(order.user_id),
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().expect(".env file not found");
    let db_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    
    let connection = sea_orm::Database::connect(&db_url).await.unwrap();
    Migrator::up(&connection, None).await.unwrap();

    let state = AppState { conn: connection };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(add_user)
            .service(add_order)
            .service(get_all_orders_for_user)
        })
        .bind(("127.0.0.1", 4765))?
        .run()
        .await
}
