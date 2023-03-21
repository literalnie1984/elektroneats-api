use actix_web::{get, web, Responder};
use chrono::Datelike;

use crate::scraper::scrape_menu;

#[get("/")]
async fn get_menu() -> actix_web::Result<impl Responder> {
    let menu = scrape_menu().await?;
    Ok(web::Json(menu))
}

#[get("/today")]
async fn get_menu_today() -> actix_web::Result<impl Responder> {
    let curr_day = chrono::offset::Local::now().date().weekday() as usize;
    let menu = scrape_menu().await?;
    Ok(web::Json(menu[curr_day].clone()))
}

#[get("/{item_id}/")]
async fn get_menu_item(item_id: web::Path<u32>) -> impl Responder {
    "TODO - display details about specific item from menu"
}
