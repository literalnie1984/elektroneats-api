use async_std::sync::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use kantyna_api::routes::menu::*;
use kantyna_api::routes::users::*;
use migration::{Migrator, MigratorTrait};

use kantyna_api::appstate::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    /* std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1"); */
    env_logger::init();

    dotenvy::dotenv().expect(".env file not found");
    let db_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");

    let connection = sea_orm::Database::connect(&db_url).await.unwrap();
    Migrator::up(&connection, None).await.unwrap();

    //create outside of closure so workers can share state
    let state = web::Data::new(AppState {
        conn: connection,
        activators: Arc::new(RwLock::new(HashMap::new())),
    });

    HttpServer::new(move || {
        let logger = Logger::default();

        App::new().wrap(logger).app_data(state.clone()).service(
            web::scope("/api")
                .service(
                    web::scope("/user")
                        .service(login)
                        .service(register)
                        .service(activate_account)
                        .service(get_user_data)
                        .service(change_password),
                )
                .service(
                    web::scope("/menu")
                        .service(get_menu)
                        .service(get_menu_item)
                        .service(get_menu_today)
                        .service(get_menu_day)
                        .service(save),
                ),
        )
    })
    .bind(("127.0.0.1", 4765))? //arbitrary port used
    .run()
    .await
}
