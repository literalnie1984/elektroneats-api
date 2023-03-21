use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use migration::{Migrator, MigratorTrait};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let connection = sea_orm::Database::connect("mysql://root:@localhost/actix_example").await.unwrap();
    Migrator::up(&connection, None).await.unwrap();
    //arbitrary port used
    HttpServer::new(|| App::new().service(hello))
        .bind(("127.0.0.1", 4765))?
        .run()
        .await
}
