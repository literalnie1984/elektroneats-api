use std::fmt::format;

use actix_web::{get, web, Responder};
use chrono::Datelike;
use entity::{dinner, extras_dinner, extras, prelude::{Dinner, Extras}};
use migration::JoinType;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, QuerySelect, RelationTrait};

use crate::{scraper::scrape_menu, errors::ServiceError, appstate::AppState};

#[get("/")]
async fn get_menu() -> actix_web::Result<impl Responder> {
    let menu = scrape_menu().await?;
    Ok(web::Json(menu))
}

// #[get("/today")]
// async fn get_menu_today() -> actix_web::Result<impl Responder> {
//     let curr_day = (chrono::offset::Local::now().date_naive().weekday() as usize).min(5);
//     let menu = scrape_menu().await?;
//     Ok(web::Json(menu[curr_day].clone()))
// }

#[get("/today")]
async fn get_menu_today(data: web::Data<AppState>) -> Result<String, ServiceError> {
    let conn = &data.conn;
    let int_to_day = |day: usize| match day {
        0 => "monday",
        1 => "tuesday",
        2 => "wednesday",
        3 => "thursday",
        4 => "friday",
        5 => "saturday",
        _ => "saturday",
    };
    let curr_day = (chrono::offset::Local::now().date_naive().weekday() as usize).min(5);
    let curr_day = int_to_day(curr_day);

    let result = Dinner::find()
        .join(JoinType::InnerJoin, extras_dinner::Relation::Extras.def())
        .filter(dinner::Column::WeekDay.eq(curr_day))
        .all(conn)
        .await.unwrap();

    Ok(format!("{:#?}", result))
}

#[get("/day/{day:[0-9]}")]
async fn get_menu_day(day: web::Path<u8>) -> actix_web::Result<impl Responder> {
    let day = day.into_inner().min(5) as usize;
    let menu = scrape_menu().await?;
    Ok(web::Json(menu[day].clone()))
}

#[get("/{item_id}/")]
async fn get_menu_item(item_id: web::Path<u32>) -> impl Responder {
    "TODO - display details about specific item from menu"
}
