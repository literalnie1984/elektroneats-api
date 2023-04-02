use actix_files::Files;
use actix_web::dev::Service;
use actix_web::http::header;
use async_std::sync::RwLock;
use kantyna_api::routes::{admin::*, menu::*, order::*, payment::*, users::*};
use std::collections::HashMap;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use migration::{Migrator, MigratorTrait};

use kantyna_api::appstate::{AppState, ClientWrapper};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if cfg!(debug_assertions) {
        std::env::set_var("RUST_LOG", "info");
        std::env::set_var("RUST_BACKTRACE", "1");
        env_logger::init();
    }

    dotenvy::dotenv().expect(".env file not found");
    let db_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let stripe_secret =
        dotenvy::var("STRIPE_SECRET").expect("STRIPE_SECRET is not set in .env file");
    let stripe_client = ClientWrapper::new(&stripe_secret);

    let connection = sea_orm::Database::connect(&db_url).await.unwrap();
    Migrator::up(&connection, None).await.unwrap();

    //create outside of closure so workers can share state
    let state = web::Data::new(AppState {
        conn: connection,
        activators_reg: Arc::new(RwLock::new(HashMap::new())),
        activators_del: Arc::new(RwLock::new(HashMap::new())),
        stripe_client,
    });

    HttpServer::new(move || {
        let logger = Logger::default();
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE]);

        let routes = web::scope("/api")
            .service(Files::new("/image", "./images"))
            .service(
                web::scope("/user")
                    .service(login)
                    .service(register)
                    .service(activate_account)
                    .service(get_user_data)
                    .service(change_password)
                    .service(get_delete_mail)
                    .service(delete_acc)
                    .service(
                        web::scope("/orders")
                            .service(create_order)
                            .service(get_completed_user_orders)
                            .service(get_pending_user_orders),
                    ),
            )
            .service(
                web::scope("/admin").service(update_dish).service(
                    web::scope("/orders")
                        .service(get_all_pending_orders)
                        .service(claim_order),
                ),
            )
            .service(
                web::scope("/payment")
                    .service(add_balance)
                    .service(init_wallet)
                    .service(get_balance)
                    .service(customer_details)
                    // .service(test_balance)
                    .service(received_payment),
            )
            .service(
                web::scope("/menu")
                    .service(get_menu_all)
                    .service(get_menu_today)
                    .service(get_menu_day)
                    .service(last_menu_update)
                    .service(update),
            );
        App::new()
            .wrap(logger)
            .wrap(cors)
            .app_data(state.clone())
            .service(routes)
    })
    .bind(("127.0.0.1", 4765))? //arbitrary port used
    .run()
    .await
}
