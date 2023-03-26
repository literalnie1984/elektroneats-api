use std::mem;

use actix_web::Responder;
use chrono::Datelike;
use entity::{
    dinner, extras,
    prelude::{Dinner, Extras, ExtrasDinner},
};
use log::error;
use paperclip::actix::{api_v2_operation, web};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, LoaderTrait, QueryFilter};

use crate::{
    appstate::AppState,
    errors::ServiceError,
    scraper::{scrape_menu, update_menu},
};

type MenuResult = Result<web::Json<Vec<(dinner::Model, Vec<extras::Model>)>>, ServiceError>;

/*
#[api_v2_operation]
async fn get_menu(conn: &DatabaseConnection, day: Option<u8>) -> MenuResult {
    let dinners = match day {
        Some(day) => Dinner::find()
            .filter(dinner::Column::WeekDay.eq(day))
            .all(conn)
            .await
            .map_err(|e| {
                error!("Database error getting menu: {}", e);
                ServiceError::InternalError
            })?,
        None => Dinner::find().all(conn).await.map_err(|e| {
            error!("Database error getting menu: {}", e);
            ServiceError::InternalError
        })?,
    };

    let mut extras = dinners
        .load_many_to_many(Extras, ExtrasDinner, conn)
        .await
        .map_err(|e| {
            error!("Database error getting menu: {}", e);
            ServiceError::InternalError
        })?;

    let response = dinners
        .iter()
        .zip(extras.iter_mut())
        .map(|(dinner, extras)| (dinner.clone(), mem::take(extras)))
        .collect::<Vec<_>>();

    Ok(web::Json(response))
}

#[api_v2_operation]
pub async fn get_menu_all(data: web::Data<AppState>) -> MenuResult {
    get_menu(&data.conn, None).await
}

#[api_v2_operation]
pub async fn get_menu_today(data: web::Data<AppState>) -> MenuResult {
    let curr_day = (chrono::offset::Local::now().date_naive().weekday() as u8).min(5);

    get_menu(&data.conn, Some(curr_day)).await
}

#[api_v2_operation]
pub async fn get_menu_day(day: web::Path<u8>, data: web::Data<AppState>) -> MenuResult {
    let day = day.into_inner().min(5) as u8;

    get_menu(&data.conn, Some(day)).await
}
*/

#[api_v2_operation]
pub async fn get_menu_item(item_id: web::Path<u32>) -> impl Responder {
    "TODO - display details about specific item from menu"
}

#[api_v2_operation]
pub async fn get_menu_update(data: web::Data<AppState>) -> Result<String, ServiceError> {
    let menu = scrape_menu().await?;
    update_menu(&data.conn, menu).await?;
    Ok("saved to db".to_owned())
}
