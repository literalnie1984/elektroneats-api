use std::{collections::HashMap, str::FromStr};

use actix_web::{get, post, web, Responder};
use entity::{prelude::User, user};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::mem;
use stripe::{self, Client, CreateCustomer, Customer, CustomerId};

use crate::{
    appstate::AppState, convert_err_to_500, errors::ServiceError, jwt_auth::AuthUser, map_db_err,
};

use super::structs::StripeUser;

#[post("/add_balance/{amount:[0-9]+}")]
async fn add_balance() -> Result<String, ServiceError> {
    todo!()
}

#[post("/pay/{amount:[0-9]+}")]
async fn pay() -> Result<String, ServiceError> {
    todo!()
}

#[get("/details")]
async fn get_customer(
    user: AuthUser,
    data: web::Data<AppState>,
) -> Result<web::Json<Customer>, ServiceError> {
    let secret_key = dotenvy::var("STRIPE_SECRET").expect("No STRIPE_SECRET variable in dotenv");
    let client = Client::new(secret_key);

    let conn = &data.conn;
    let user = User::find_by_id(user.id)
        .one(conn)
        .await
        .map_err(map_db_err)?;
    let Some(user) = user else {return Err(ServiceError::BadRequest("No user has given id".into()))};
    if let Some(user_id) = user.stripe_id {
        let id: CustomerId = CustomerId::from_str(&user_id).unwrap();
        let customer = Customer::retrieve(&client, &id, &[])
            .await
            .map_err(|e| convert_err_to_500(e, Some("Stripe err")))?;
        Ok(web::Json(customer))
    } else {
        Err(ServiceError::BadRequest(
            "Stripe wasn't initialized for provided user".into(),
        ))
    }
}

#[post("/init")]
async fn init_wallet(
    user: AuthUser,
    data: web::Data<AppState>,
    stripe_data: web::Json<StripeUser>,
) -> Result<String, ServiceError> {
    create_customer(&data.conn, user.id, stripe_data.0).await?;
    Ok("Success".into())
}

pub async fn create_customer(
    conn: &DatabaseConnection,
    user_id: i32,
    mut stripe_data: StripeUser,
) -> Result<(), ServiceError> {
    let secret_key = dotenvy::var("STRIPE_SECRET").expect("No STRIPE_SECRET variable in dotenv");
    let client = Client::new(secret_key);

    let user = User::find_by_id(user_id)
        .one(conn)
        .await
        .map_err(map_db_err)?;

    let Some(user) = user else {return Err(ServiceError::BadRequest("No user has given id".into()))};
    if user.stripe_id.is_some() {
        return Err(ServiceError::BadRequest(
            "Provided user is already registered in stripe".into(),
        ));
    }

    let addr = &mut stripe_data.address;
    let customer = Customer::create(
        &client,
        CreateCustomer {
            email: Some(&user.email),
            name: Some(&stripe_data.name),
            phone: Some(&stripe_data.phone),
            address: Some(stripe::Address {
                city: Some(mem::take(&mut addr.city)),
                country: Some(mem::take(&mut addr.country)),
                postal_code: Some(mem::take(&mut addr.postal_code)),
                state: Some(mem::take(&mut addr.state)),
                ..Default::default()
            }),
            metadata: Some(HashMap::from([("async-stripe".into(), "true".into())])),
            ..Default::default()
        },
    )
    .await
    .map_err(|e| convert_err_to_500(e, Some("Stripe error")))?;

    let mut user_upd: user::ActiveModel = user.into();
    user_upd.stripe_id = Set(Some(customer.id.to_string()));
    user_upd.update(conn).await.map_err(map_db_err)?;
    Ok(())
}
