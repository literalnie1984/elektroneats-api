use std::str::FromStr;

use actix_web::{get, post, web, Responder};
use entity::{prelude::User, user};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use stripe::{self, Client, CreateCustomer, Customer, CustomerId};

use crate::{
    appstate::AppState, convert_err_to_500, errors::ServiceError, jwt_auth::AuthUser, map_db_err,
};

#[post("/add_balance/{amount:[0-9]+}")]
async fn add_balance() -> Result<String, ServiceError> {
    todo!()
}

#[get("/check_balance")]
async fn check_balance() -> Result<String, ServiceError> {
    todo!()
}

#[post("/pay/{amount:[0-9]+}")]
async fn pay() -> Result<String, ServiceError> {
    todo!()
}

#[post("/init")]
async fn init_wallet(
    user: AuthUser,
    data: web::Data<AppState>,
    amount: web::Path<u32>,
) -> Result<impl Responder, ServiceError> {
    let amount = amount.into_inner();
    let secret_key = dotenvy::var("STRIPE_SECRET").expect("No STRIPE_SECRET variable in dotenv");
    let client = Client::new(secret_key);
    let conn = &data.conn;

    let customer = get_or_create_customer(conn, user.id, &client).await?;
    Ok(web::Json(customer))
}

pub async fn get_or_create_customer(
    conn: &DatabaseConnection,
    user_id: i32,
    client: &stripe::Client,
) -> Result<Customer, ServiceError> {
    let user = User::find_by_id(user_id)
        .one(conn)
        .await
        .map_err(map_db_err)?;
    let Some(user) = user else {return Err(ServiceError::BadRequest("No user has given id".into()))};
    if let Some(user_id) = user.stripe_id {
        eprintln!("retrieving");
        let id: CustomerId = CustomerId::from_str(&user_id).unwrap();
        let customer = Customer::retrieve(client, &id, &[])
            .await
            .map_err(|e| convert_err_to_500(e, Some("Stripe err")))?;
        Ok(customer)
    } else {
        let customer = Customer::create(
            &client,
            CreateCustomer {
                name: Some(&user.username),
                email: Some(&user.email),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| convert_err_to_500(e, Some("Stripe error")))?;

        let mut user_upd: user::ActiveModel = user.into();
        user_upd.stripe_id = Set(Some(customer.id.to_string()));
        user_upd.update(conn).await.map_err(map_db_err)?;
        Ok(customer)
    }
}
