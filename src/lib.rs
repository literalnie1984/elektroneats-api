use crate::scraper::{insert_static_extras, scrape_menu, update_menu};
use actix_web::HttpRequest;
use appstate::ActivatorsVec;
use entity::prelude::User;
use enums::VerificationType;
use jwt_auth::AuthUser;
use lettre::{
    transport::smtp::{
        authentication::{Credentials, Mechanism},
        PoolConfig,
    },
    AsyncSmtpTransport, AsyncStd1Executor, AsyncTransport,
};
use log::{error, info};
use migration::DbErr;
use nanoid::nanoid;
use sea_orm::{DatabaseConnection, EntityTrait};
use std::{fmt::Display, str::FromStr, thread};
use stripe::{Client, Customer, CustomerId, UpdateCustomer};

use errors::ServiceError;

pub mod appstate;
pub mod enums;
pub mod errors;
pub mod jwt_auth;
pub mod routes;
pub mod scraper;

const CODE_INTS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

pub fn convert_err_to_500<E>(err: E, msg: Option<&str>) -> ServiceError
where
    E: Display,
{
    let msg = msg.unwrap_or("Error");
    error!("{}: {}", msg, err);
    ServiceError::InternalError
}

pub fn map_db_err(err: DbErr) -> ServiceError {
    err.into()
}

#[macro_export]
macro_rules! update_if_some {
    ($db: expr, $new: expr) => {
        if let Some(new) = $new {
            $db = Set(new);
        }
    };
}

pub async fn send_verification_mail(
    email: &str,
    activators: &ActivatorsVec,
    email_type: VerificationType,
) -> Result<String, ServiceError> {
    let from = "Kantyna-App <kantyna.noreply@mikut.dev>".parse().unwrap();
    let to = email
        .parse()
        .map_err(|err| convert_err_to_500(err, Some("Mail creation err")))?;

    //add email - activation_link combo to current app state
    let code_len = email_type.code_len();
    let activation_code = nanoid!(code_len, &CODE_INTS);
    let mail = email_type
        .email_msg(to, from, &activation_code)
        .map_err(|err| convert_err_to_500(err, Some("Mail creation err")))?;
    let mut activators = activators.write().await;
    (*activators).insert(activation_code, email.into());

    let smtp: AsyncSmtpTransport<AsyncStd1Executor> =
        AsyncSmtpTransport::<AsyncStd1Executor>::starttls_relay("mikut.dev")
            .unwrap()
            .credentials(Credentials::new(
                "kantyna.noreply@mikut.dev".to_owned(),
                dotenvy::var("EMAIL_PASS").expect("NO EMAIL_PASS val provided in .env"),
            ))
            .authentication(vec![Mechanism::Plain])
            .pool_config(PoolConfig::new().max_size(20))
            .build();

    actix_rt::spawn(async move {
        info!("robimy mały trolling");
        for _i in 0..2{
            match smtp.send(mail.clone()).await{
                Ok(_) => break,
                Err(e) => error!("Mail send error: {e}"),
            }
        }
    });

    Ok("email send".to_string())
}

pub fn get_header_val<'r>(req: &'r HttpRequest, key: &'r str) -> Option<&'r str> {
    req.headers().get(key)?.to_str().ok()
}

pub async fn pay(
    client: &stripe::Client,
    conn: &DatabaseConnection,
    user: AuthUser,
    amount: i64,
) -> Result<String, ServiceError> {
    let decr = amount;

    let (customer_id, old_balance) = {
        let customer = get_user(conn, user.id, client).await?;

        (customer.id, customer.balance.unwrap())
    };

    Customer::update(
        client,
        &customer_id,
        UpdateCustomer {
            balance: Some(old_balance - decr),
            ..Default::default()
        },
    )
    .await
    .map_err(|e| convert_err_to_500(e, Some("Stripe error")))?;

    Ok("Success".into())
}

pub async fn get_user(
    conn: &DatabaseConnection,
    user_id: i32,
    client: &Client,
) -> Result<Customer, ServiceError> {
    let user = User::find_by_id(user_id)
        .one(conn)
        .await
        .map_err(map_db_err)?;

    let Some(user) = user else {return Err(ServiceError::BadRequest("No user has given id".into()))};
    if let Some(user_id) = user.stripe_id {
        let id: CustomerId = CustomerId::from_str(&user_id).unwrap();
        let customer = Customer::retrieve(client, &id, &[])
            .await
            .map_err(|e| convert_err_to_500(e, Some("Stripe err")))?;
        Ok(customer)
    } else {
        Err(ServiceError::BadRequest(
            "Stripe wasn't initialized for provided user".into(),
        ))
    }
}

pub async fn init_db(conn: &DatabaseConnection) -> Result<(), ServiceError> {
    let menu = scrape_menu().await?;
    insert_static_extras(conn).await?;
    update_menu(conn, menu).await?;
    Ok(())
}
