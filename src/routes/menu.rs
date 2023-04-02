use std::mem;

use actix_web::{get, web, Responder};
use chrono::Datelike;
use entity::{
    custom_impl::DinnerToExtras,
    dinner,
    prelude::{Dinner, Extras, ExtrasDinner},
};
use sea_orm::ModelTrait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, LoaderTrait, QueryFilter, QueryOrder};

use crate::{
    appstate::AppState,
    errors::ServiceError,
    map_db_err,
    routes::structs::MenuOneDay,
    scraper::{scrape_menu, update_menu},
};

use super::structs::DinnerWithExtras;

type MenuResult3d = Result<web::Json<Vec<Vec<DinnerWithExtras>>>, ServiceError>;
type MenuResult = Result<web::Json<MenuOneDay>, ServiceError>;

async fn get_menu(conn: &DatabaseConnection, day: u8) -> MenuResult {
    let dinners = Dinner::find()
        .filter(dinner::Column::WeekDay.eq(day))
        .all(conn)
        .await
        .map_err(map_db_err)?;

    if dinners.is_empty() {
        return Err(ServiceError::NotFound("No dinners exists".to_string()));
    }

    let extras = dinners[0]
        .find_linked(DinnerToExtras)
        .all(conn)
        .await
        .map_err(map_db_err)?;

    // let extras = Extras::find()
    //     .from_raw_sql(
    //         Statement::from_string(DbBackend::MySql,
    //             format!(r#"select e.* from extras e join extras_dinner ed on ed.extras_id=e.id where ed.dinner_id = {};"#,dinner_day_id)))
    //     .all(conn).await.map_err(map_db_err)?;
    //REWRITE THIS IN SEAORM, ONLY HERE IN THIS STATE TEMPORARILY

    /* let mut extras = dinners
    .load_many_to_many(Extras, ExtrasDinner, conn)
    .await
    .map_err(map_db_err)?; */

    /* let response = dinners
    .iter()
    .zip(extras.iter_mut())
    .map(|(dinner, extras)| (dinner.clone(), mem::take(extras)))
    .collect::<Vec<_>>(); */

    Ok(web::Json(MenuOneDay { dinners, extras }))
}

async fn get_menu_3d(conn: &DatabaseConnection) -> MenuResult3d {
    let mut dinners = Dinner::find()
        .order_by_asc(dinner::Column::WeekDay)
        .all(conn)
        .await
        .map_err(map_db_err)?;

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
        }
        dinner_day.push(DinnerWithExtras {
            dinner: mem::take(dinner),
            extras: mem::take(extras),
        });
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

    get_menu(&data.conn, curr_day).await
}

#[get("/day/{day:[0-9]}")]
async fn get_menu_day(day: web::Path<u8>, data: web::Data<AppState>) -> MenuResult {
    let day = day.into_inner().min(5) as u8;

    get_menu(&data.conn, day).await
}

#[get("/update")]
async fn update(data: web::Data<AppState>) -> Result<impl Responder, ServiceError> {
    let menu = scrape_menu().await?;
    update_menu(&data.conn, menu).await?;
    Ok("saved to db")
}
