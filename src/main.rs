use actix_web::{web, App, HttpServer};
use kantyna_api::routes::menu::*;
use kantyna_api::routes::users::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(
            web::scope("/api")
                .service(
                    web::scope("/user")
                        .service(login)
                        .service(register)
                        .service(is_logged),
                )
                .service(web::scope("/menu").service(get_menu).service(get_menu_item)),
        )
    })
    .bind(("127.0.0.1", 4765))? //arbitrary port used
    .run()
    .await
}
