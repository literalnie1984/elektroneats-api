use appstate::ActivatorsVec;
use entity::prelude::User;
use entity::user;
use enums::VerificationType;
use lettre::{
    transport::smtp::{
        authentication::{Credentials, Mechanism},
        PoolConfig,
    },
    AsyncSmtpTransport, AsyncStd1Executor, AsyncTransport,
};
use log::error;
use migration::DbErr;
use nanoid::nanoid;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::{fmt::Display, str::FromStr};
use stripe::{self, CreateCustomer, Customer, CustomerId};

use errors::ServiceError;

pub mod appstate;
pub mod enums;
pub mod errors;
pub mod jwt_auth;
pub mod routes;
pub mod scraper;

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
    let activation_link = nanoid!();
    let mail = email_type
        .email_msg(to, from, &activation_link)
        .map_err(|err| convert_err_to_500(err, Some("Mail creation err")))?;
    let mut activators = activators.write().await;
    (*activators).insert(activation_link, email.into());

    let smtp: AsyncSmtpTransport<AsyncStd1Executor> =
        AsyncSmtpTransport::<AsyncStd1Executor>::starttls_relay("mikut.dev")
            .unwrap()
            .credentials(Credentials::new(
                "kantyna.noreply@mikut.dev".to_owned(),
                dotenvy::var("EMAIL_PASS")
                    .expect("NO EMAIL_PASS val provided in .env")
                    .to_string(),
            ))
            .authentication(vec![Mechanism::Plain])
            .pool_config(PoolConfig::new().max_size(20))
            .build();

    match smtp.send(mail).await {
        Err(_) => Err(ServiceError::InternalError),
        Ok(_) => Ok("email send".to_string()),
    }
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
