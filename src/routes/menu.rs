use std::mem;

use crate::routes::structs;
use actix_web::Responder;
use chrono::Datelike;
use entity::{
    dinner, extras,
    prelude::{Dinner, Extras, ExtrasDinner},
};
use log::error;
use paperclip::actix::{api_v2_operation, web, Apiv2Schema};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, LoaderTrait, QueryFilter};
use serde::Serialize;

use crate::{
    appstate::AppState,
    errors::ServiceError,
    scraper::{scrape_menu, update_menu},
};

#[derive(Apiv2Schema, Serialize)]
pub struct MenuVec(Vec<MenuVecInner>);
#[derive(Apiv2Schema, Serialize)]
pub struct MenuVecInner(structs::Dinner, Vec<structs::Extras>);
type MenuResult = Result<web::Json<MenuVec>, ServiceError>;

async fn get_menu(conn: &DatabaseConnection, day: Option<u8>) -> MenuResult {
    fn dinner_model_to_dinner(dinner_model: dinner::Model) -> structs::Dinner {
        structs::Dinner {
            id: dinner_model.id,
            name: dinner_model.name,
            price: dinner_model.price.to_string().parse().unwrap(),
            image: dinner_model.image,
            week_day: dinner_model.week_day,
            max_supply: dinner_model.max_supply,
            r#type: dinner_model.r#type.into(),
        }
    }

    fn extras_model_to_extras(extras_model: extras::Model) -> structs::Extras {
        structs::Extras {
            id: extras_model.id,
            name: extras_model.name,
            price: extras_model.price.to_string().parse().unwrap(),
        }
    }

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

    let response: MenuVec = MenuVec(
        dinners
            .iter()
            .zip(extras.iter_mut())
            .map(|(dinner, extras)| {
                let dinners = dinner_model_to_dinner(dinner.clone());
                let extras: Vec<structs::Extras> = extras
                    .iter()
                    .map(|e| extras_model_to_extras(e.clone()))
                    .collect();
                MenuVecInner(dinners, extras)
            })
            .collect::<Vec<_>>(),
    );

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
