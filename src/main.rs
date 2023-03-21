use actix_web::{web, App, HttpServer};
use kantyna_api::routes::menu::*;
use kantyna_api::routes::users::*;
use kantyna_api::AppState;
use migration::{Migrator, MigratorTrait};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().expect(".env file not found");
    let db_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");

    let connection = sea_orm::Database::connect(&db_url).await.unwrap();
    Migrator::up(&connection, None).await.unwrap();

    let state = AppState { conn: connection };

    HttpServer::new(move || {
        App::new().app_data(web::Data::new(state.clone())).service(
            web::scope("/api")
                .service(
                    web::scope("/user")
                        .service(add_user)
                        .service(add_order)
                        .service(get_all_orders_for_user),
                )
                .service(
                    web::scope("/menu")
                        .service(get_menu)
                        .service(get_menu_item)
                        .service(get_menu_today)
                        .service(get_menu_day),
                ),
        )
    })
    .bind(("127.0.0.1", 4765))? //arbitrary port used
    .run()
    .await
}
