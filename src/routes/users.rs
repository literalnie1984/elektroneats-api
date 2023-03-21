use actix_web::{get, post, web, Responder};

#[post("/login")]
async fn login(data: web::Json<u32>) -> impl Responder {
    "TODO - login with given data"
}

#[post("/register")]
async fn register(data: web::Json<u32>) -> impl Responder {
    "TODO - reigster with given data"
}

#[get("/is_logged")]
async fn is_logged() -> impl Responder {
    "TODO"
}
