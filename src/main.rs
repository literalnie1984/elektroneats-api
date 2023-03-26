use async_std::sync::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
use kantyna_api::routes::menu::*;
use kantyna_api::routes::users::*;
use migration::{Migrator, MigratorTrait};
use paperclip::actix::{
    web::{self, Json},
    OpenApiExt,
};

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
        activators_reg: Arc::new(RwLock::new(HashMap::new())),
        activators_del: Arc::new(RwLock::new(HashMap::new())),
    });

    HttpServer::new(move || {
        let logger = Logger::default();

        App::new()
            .wrap_api()
            .wrap(logger)
            .app_data(state.clone())
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/user")
                            .service(web::resource("/login").route(web::post().to(login)))
                            .service(web::resource("/register").route(web::post().to(register)))
                            .service(
                                web::resource("/activate/{token}")
                                    .route(web::get().to(activate_account)),
                            )
                            .service(
                                web::resource("/get-user-data").route(web::get().to(get_user_data)),
                            )
                            .service(
                                web::resource("/change-password")
                                    .route(web::post().to(change_password)),
                            )
                            .service(web::resource("/delete").route(web::get().to(get_delete_mail)))
                            .service(
                                web::resource("/delete/{token}").route(web::get().to(delete_acc)),
                            ),
                    )
                    .service(
                        web::scope("/menu")
                            .service(web::resource("/").route(web::get().to(get_menu_all)))
                            .service(web::resource("/today").route(web::get().to(get_menu_today)))
                            .service(
                                web::resource("/{item_id}").route(web::get().to(get_menu_item)),
                            )
                            .service(
                                web::resource("/day/{day:[0-9]}")
                                    .route(web::get().to(get_menu_day)),
                            )
                            .service(
                                web::resource("/update").route(web::get().to(get_menu_update)),
                            ),
                    ),
            )
            .with_json_spec_at("/openapi/v2/spec")
            .build()
    })
    .bind(("127.0.0.1", 4765))? //arbitrary port used
    .run()
    .await
}
