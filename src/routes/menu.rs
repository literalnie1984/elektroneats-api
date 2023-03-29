use std::mem;

use actix_web::{get, web, Responder};
use chrono::Datelike;
use entity::{
    dinner, extras,
    prelude::{Dinner, Extras, ExtrasDinner},
};
use log::error;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, LoaderTrait, QueryFilter};

use crate::{
    appstate::AppState,
    errors::ServiceError,
    map_db_err,
    scraper::{scrape_menu, update_menu},
};

type MenuResult = Result<web::Json<Vec<(dinner::Model, Vec<extras::Model>)>>, ServiceError>;

async fn get_menu(conn: &DatabaseConnection, day: Option<u8>) -> MenuResult {
    let dinners = match day {
        Some(day) => Dinner::find()
            .filter(dinner::Column::WeekDay.eq(day))
            .all(conn)
            .await
            .map_err(map_db_err)?,
        None => Dinner::find().all(conn).await.map_err(map_db_err)?,
    };

    let mut extras = dinners
        .load_many_to_many(Extras, ExtrasDinner, conn)
        .await
        .map_err(map_db_err)?;

    let response = dinners
        .iter()
        .zip(extras.iter_mut())
        .map(|(dinner, extras)| (dinner.clone(), mem::take(extras)))
        .collect::<Vec<_>>();

    Ok(web::Json(response))
}

#[get("/")]
async fn get_menu_all(data: web::Data<AppState>) -> MenuResult {
    get_menu(&data.conn, None).await
}

#[get("/today")]
async fn get_menu_today(data: web::Data<AppState>) -> MenuResult {
    let curr_day = (chrono::offset::Local::now().date_naive().weekday() as u8).min(5);

    get_menu(&data.conn, Some(curr_day)).await
}

#[get("/day/{day:[0-9]}")]
async fn get_menu_day(day: web::Path<u8>, data: web::Data<AppState>) -> MenuResult {
    let day = day.into_inner().min(5) as u8;

    get_menu(&data.conn, Some(day)).await
}

#[get("/{item_id}/")]
async fn get_menu_item(item_id: web::Path<u32>) -> impl Responder {
    "TODO - display details about specific item from menu"
}

#[get("/update")]
async fn update(data: web::Data<AppState>) -> Result<impl Responder, ServiceError> {
    let menu = scrape_menu().await?;
    update_menu(&data.conn, menu).await?;
    Ok("saved to db")
}
