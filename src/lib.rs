use actix_web::HttpRequest;
use appstate::ActivatorsVec;
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
use std::fmt::Display;

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
