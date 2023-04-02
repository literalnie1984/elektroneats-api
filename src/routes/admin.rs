use actix_web::{get, put, web};
use entity::{
    dinner, dinner_orders,
    prelude::{Dinner, DinnerOrders}, model_enums::Status,
};
use sea_orm::{prelude::Decimal, ActiveModelTrait, EntityTrait, Set, ActiveEnum};
use serde::{Serialize, Deserialize};
use std::mem;

use crate::{
    appstate::AppState, errors::ServiceError, jwt_auth::AuthUser, map_db_err, update_if_some,
};

use super::structs::UpdateMenu;

#[put("/update-dish")]
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

    //TODO: write better macro for this
    update_if_some!(selected_dish.name, new_dish.name);
    update_if_some!(selected_dish.image, new_dish.image);
    update_if_some!(selected_dish.max_supply, new_dish.max_supply);
    if let Some(price) = new_dish.price {
        selected_dish.price = Set(Decimal::from_f32_retain(price).unwrap());
    }
    if let Some(week_day) = new_dish.week_day {
        selected_dish.week_day = Set(week_day as u8);
    }
    selected_dish.update(conn).await.map_err(map_db_err)?;

    Ok("Success".into())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusRequest{
    pub new_status: Status,
}

#[put("/{id}/status")]
async fn change_order_status(
    user: AuthUser,
    path: web::Path<i32>,
    data: web::Data<AppState>,
    body: web::Json<StatusRequest>
) -> Result<String, ServiceError> {
    // if !user.is_admin {
    //     return Err(ServiceError::Unauthorized(
    //         "You need to be an admin to access this".into(),
    //     ));
    // }

    let conn = &data.conn;
    let claim_id = path.into_inner();

    let mut order: dinner_orders::ActiveModel = {
        let order = DinnerOrders::find_by_id(claim_id)
            .one(conn)
            .await
            .map_err(map_db_err)?;
        let Some(order) = order else {return Err(ServiceError::BadRequest("Invalid dinner_order id".into()))};
        order.into()
    };

    //MySQL has no bools
    order.status = Set(body.into_inner().new_status.into_value());
    order.update(conn).await.map_err(map_db_err)?;

    Ok("Success".into())
}
