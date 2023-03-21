use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //arbitrary port used
    HttpServer::new(|| App::new().service(hello))
        .bind(("127.0.0.1", 4765))?
        .run()
        .await
}
