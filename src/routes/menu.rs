use actix_web::{get, web, Responder};

#[get("/")]
async fn get_menu() -> impl Responder {
    "TODO - get menu for today"
}

#[get("/{item_id}/")]
async fn get_menu_item(item_id: web::Path<u32>) -> impl Responder {
    "TODO - display details about specific item from menu"
}
