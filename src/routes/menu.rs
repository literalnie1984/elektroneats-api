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

type MenuResult3d = Result<web::Json<Vec<Vec<(dinner::Model, Vec<extras::Model>)>>>, ServiceError>;
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

async fn get_menu_3d(conn: &DatabaseConnection) -> MenuResult3d {
    let mut dinners = Dinner::find().all(conn).await.map_err(map_db_err)?;

    let mut extras = dinners
        .load_many_to_many(Extras, ExtrasDinner, conn)
        .await
        .map_err(map_db_err)?;

    let mut response = Vec::with_capacity(6);
    let mut last_day = 0;
    let mut dinner_day = Vec::with_capacity(4);
    for (dinner, extras) in dinners.iter_mut().zip(extras.iter_mut()) {
        if dinner.week_day != last_day {
            response.push(dinner_day);
            dinner_day = Vec::with_capacity(4);
            last_day = dinner.week_day;
        } else {
            dinner_day.push((mem::take(dinner), mem::take(extras)));
        }
    }
    response.push(dinner_day);

    Ok(web::Json(response))
}

#[get("/")]
async fn get_menu_all(data: web::Data<AppState>) -> MenuResult3d {
    get_menu_3d(&data.conn).await
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

#[get("/update")]
async fn update(data: web::Data<AppState>) -> Result<impl Responder, ServiceError> {
    let menu = scrape_menu().await?;
    update_menu(&data.conn, menu).await?;
    Ok("saved to db")
}
