use std::{fmt::format, mem};

use actix_web::{get, web, Responder};
use chrono::Datelike;
use entity::{dinner, extras_dinner, extras, prelude::{Dinner, Extras, ExtrasDinner}};
use log::info;
use migration::JoinType;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, QuerySelect, RelationTrait, LoaderTrait};

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
async fn get_menu_today(data: web::Data<AppState>) -> Result<web::Json<Vec<(dinner::Model, Vec<extras::Model>)>>, ServiceError> {
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

    let dinners = Dinner::find().filter(dinner::Column::WeekDay.eq(curr_day)).all(conn).await.unwrap();
    let mut extras  = dinners.load_many_to_many(Extras, ExtrasDinner, conn).await.unwrap();

    let response = dinners.iter().zip(extras.iter_mut())
        .map(|(dinner, extras)| {
            (dinner.clone(), mem::take(extras))
        }).collect::<Vec<_>>();

    Ok(web::Json(response))
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
