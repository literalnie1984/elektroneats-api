use std::{collections::HashSet, mem};

use crate::{jwt_auth::AuthUser, routes::structs::{MenuResult3D, LastUpdateResponse}};
use actix_web::{get, web, Responder};
use chrono::{DateTime, Datelike, Utc};
use entity::{
    custom_impl::DinnerToExtras,
    dinner, menu_info,
    prelude::{Dinner, Extras, ExtrasDinner},
};
use sea_orm::ModelTrait;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, LoaderTrait, QueryFilter, QueryOrder, QuerySelect,
};

use crate::{
    appstate::AppState,
    errors::ServiceError,
    map_db_err,
    routes::structs::MenuOneDay,
    scraper::{scrape_menu, update_menu},
};

use super::structs::DinnerWithExtras;

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

async fn get_menu_3d(conn: &DatabaseConnection) -> Result<web::Json<MenuResult3D>, ServiceError> {
    let mut dinners = Dinner::find()
        .order_by(dinner::Column::WeekDay, migration::Order::Asc)
        .all(conn)
        .await
        .map_err(map_db_err)?;

    let extras = dinners
        .load_many_to_many(Extras, ExtrasDinner, conn)
        .await
        .map_err(map_db_err)?;

    let mut result = MenuResult3D {
        response: vec![
            DinnerWithExtras {
                dinners: Vec::with_capacity(4),
                extras_ids: Vec::new()
            };
            6
        ],
        extras: HashSet::new(),
    };
    for (dinner, extras) in dinners.iter_mut().zip(extras.iter()) {
        let index = dinner.week_day as usize;

        result.extras.extend(extras.clone());
        result.response[index].dinners.push(mem::take(dinner));
        if result.response[index].extras_ids.is_empty() {
            result.response[index].extras_ids = extras.iter().map(|x| x.id).collect();
        }

        /* push(DinnerWithExtras {
            dinner: mem::take(dinner),
            extras_ids: extras.iter().map(|x| x.id).collect(),
        }); */
    }

    Ok(web::Json(result))
}

#[get("/")]
async fn get_menu_all(data: web::Data<AppState>) -> Result<web::Json<MenuResult3D>, ServiceError> {
    get_menu_3d(&data.conn).await
}

#[get("/today")]
async fn get_menu_today(data: web::Data<AppState>) -> MenuResult {
    let curr_day = (chrono::offset::Local::now().date_naive().weekday() as u8).min(5);

    get_menu(&data.conn, curr_day).await
}

#[get("/day/{day:[0-9]}")]
async fn get_menu_day(day: web::Path<u8>, data: web::Data<AppState>) -> MenuResult {
    let day = day.into_inner().min(5);

    get_menu(&data.conn, day).await
}

#[get("/last-update")]
async fn last_menu_update(data: web::Data<AppState>) -> Result<impl Responder, ServiceError> {
    let date: DateTime<Utc> = menu_info::Entity::find()
        .select_only()
        .column(menu_info::Column::LastUpdate)
        .into_tuple()
        .one(&data.conn)
        .await
        .map_err(map_db_err)?
        .unwrap();

    Ok(web::Json(LastUpdateResponse{ last_update: date }))
}

#[get("/update")]
async fn update(data: web::Data<AppState>, user: AuthUser) -> Result<String, ServiceError> {
    if !user.is_admin {
        return Err(ServiceError::Unauthorized(
            "Only admin can access that data".to_string(),
        ));
    }

    let menu = scrape_menu().await?;
    update_menu(&data.conn, menu).await?;
    Ok("Success".into())
}
