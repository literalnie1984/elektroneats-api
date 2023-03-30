use actix_web::{put, web};
use entity::{dinner, prelude::Dinner};
use sea_orm::{prelude::Decimal, ActiveModelTrait, EntityTrait, Set};
use std::mem;

use crate::{appstate::AppState, errors::ServiceError, jwt_auth::AuthUser, map_db_err};

use super::structs::UpdateMenu;

#[put("/update_dish")]
async fn update_dish(
    user: AuthUser,
    data: web::Data<AppState>,
    mut new_dish: web::Json<UpdateMenu>,
) -> Result<String, ServiceError> {
    if !user.is_admin {
        return Err(ServiceError::Unauthorized(
            "You need to be an admin to access this".into(),
        ));
    }

    let new_dish = mem::take(&mut new_dish.0);
    let conn = &data.conn;

    let selected_dish = Dinner::find_by_id(new_dish.id)
        .one(conn)
        .await
        .map_err(map_db_err)?;

    let mut selected_dish: dinner::ActiveModel = {
        let Some(selected_dish) = selected_dish else {return Err(ServiceError::BadRequest("No dish has given id".into()))};
        selected_dish.into()
    };

    //TODO: write macro for this
    if let Some(name) = new_dish.name {
        selected_dish.name = Set(name);
    }
    if let Some(price) = new_dish.price {
        selected_dish.price = Set(Decimal::from_f32_retain(price).unwrap());
    }
    if let Some(image) = new_dish.image {
        selected_dish.image = Set(image);
    }
    if let Some(max_supply) = new_dish.max_supply {
        selected_dish.max_supply = Set(max_supply);
    }
    if let Some(week_day) = new_dish.week_day {
        selected_dish.week_day = Set(week_day as u8);
    }
    selected_dish.update(conn).await.map_err(map_db_err)?;

    Ok("Success".into())
}
