use actix_web::web::Path;
use actix_web::{get, post, web, Responder};
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::transport::smtp::PoolConfig;
use lettre::{AsyncSmtpTransport, AsyncStd1Executor, AsyncTransport};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, Set};

use bcrypt::{hash_with_salt, verify, DEFAULT_COST};
use nanoid::nanoid;

use entity::prelude::User;
use entity::user;

use crate::appstate::{ActivatorsVec, AppState};
use crate::convert_err_to_500;
use crate::enums::VerificationType;

use crate::errors::ServiceError;
use crate::jwt_auth::create_jwt;
use crate::jwt_auth::AuthUser;
use crate::routes::structs::{UserChangePassword, UserLogin, UserRegister};

use log::error;

#[post("/change-password")]
async fn change_password(
    user: AuthUser,
    data: web::Data<AppState>,
    pass_data: web::Json<UserChangePassword>,
) -> Result<String, ServiceError> {
    let conn = &data.conn;

    let user_query = User::find()
        .filter(user::Column::Id.eq(user.id))
        .one(conn)
        .await
        .map_err(|err| convert_err_to_500(err, Some("Database error")))?;

    let Some(user) = user_query else {return Err(ServiceError::BadRequest( "Account does not exist".to_string(),))};

    if !verify(&pass_data.old_password, &user.password).unwrap() {
        return Err(ServiceError::BadRequest(
            "Old password is incorrect".to_string(),
        ));
    }

    let salt = nanoid!(16);
    let salt_copy: [u8; 16] = salt.as_bytes().try_into().unwrap();
    let new_password = hash_with_salt(&pass_data.new_password, DEFAULT_COST, salt_copy).unwrap();

    let mut user: user::ActiveModel = user.into();
    user.password = Set(new_password.to_string());
    match user.update(conn).await {
        Ok(_) => Ok("Password changed".to_string()),
        Err(error) => {
            error!("Database error: {}", error);
            return Err(ServiceError::InternalError);
        }
    }
}

#[get("/get-user-data")]
async fn get_user_data(user: AuthUser, data: web::Data<AppState>) -> impl Responder {
    let conn = &data.conn;

    let user_query = User::find()
        .filter(user::Column::Id.eq(user.id))
        .one(conn)
        .await
        .map_err(|err| convert_err_to_500(err, Some("Database error")))?;

    let Some(user) = user_query else {return Err(ServiceError::BadRequest("Account does not exist".into()))};

    Ok(format!("User data: {}", user.username))
}

#[get("/delete")]
async fn get_delete_mail(
    user: AuthUser,
    data: web::Data<AppState>,
) -> Result<impl Responder, ServiceError> {
    let conn = &data.conn;

    let user_query = User::find()
        .filter(user::Column::Id.eq(user.id))
        .one(conn)
        .await
        .map_err(|err| convert_err_to_500(err, Some("Database error")))?;

    let Some(user) = user_query else {return Err(ServiceError::BadRequest("Account does not exist".into()))};

    send_verification_mail(&user.email, &data.activators_del, VerificationType::Delete).await
}

#[get("/delete/{token}")]
async fn delete_acc(
    _user: AuthUser,
    data: web::Data<AppState>,
    token: Path<String>,
) -> Result<impl Responder, ServiceError> {
    let tokens = data.activators_del.read().await;
    let Some(email) = tokens.get(&token.into_inner()) else {return Err(ServiceError::BadRequest("Invalid deletion token!".into()))};
    let conn = &data.conn;
    let user_query: Option<user::Model> = User::find()
        .filter(user::Column::Email.eq(email))
        .one(conn)
        .await
        .map_err(|err| convert_err_to_500(err, Some("Database error")))?;

    let Some(user) = user_query else {return Err(ServiceError::InternalError)};

    let res = user
        .delete(conn)
        .await
        .map_err(|err| convert_err_to_500(err, Some("Database error")))?;

    if res.rows_affected == 1 {
        Ok("Deleted account successfully")
    } else {
        Err(ServiceError::InternalError)
    }
}

#[post("/login")]
async fn login(
    user: web::Json<UserLogin>,
    data: web::Data<AppState>,
) -> Result<String, ServiceError> {
    let conn = &data.conn;
    let user = user.into_inner();
    let user_query = User::find()
        .filter(user::Column::Email.eq(user.email))
        .one(conn)
        .await
        .map_err(|err| convert_err_to_500(err, Some("Database error")))?;

    let Some(user) = user_query else {return Err(ServiceError::BadRequest("Account does not exist".into()))};
    let result = verify(&user.password, &user.password).unwrap();

    if result {
        let token = match create_jwt(user.id) {
            Ok(token) => token,
            Err(error) => {
                eprintln!("Error creating token: {}", error);
                return Err(ServiceError::InternalError);
            }
        };

        Ok(token)
    } else {
        Err(ServiceError::Unauthorized(
            "Invalid credentials".to_string(),
        ))
    }
}

#[post("/register")]
async fn register(user: web::Json<UserRegister>, data: web::Data<AppState>) -> impl Responder {
    let conn = &data.conn;

    let user = user.into_inner();

    let user_query = User::find()
        .filter(user::Column::Email.eq(&user.email))
        .one(conn)
        .await
        .map_err(|err| convert_err_to_500(err, Some("Database error")))?;

    if user_query.is_some() {
        return Err(ServiceError::BadRequest(
            "Account already exists".to_string(),
        ));
    }

    let salt = nanoid!(16);
    let salt_copy: [u8; 16] = salt.as_bytes().try_into().unwrap();
    let hashed_pass = hash_with_salt(user.password.as_bytes(), DEFAULT_COST, salt_copy).unwrap();

    user::ActiveModel {
        username: Set(user.username),
        email: Set(user.email.clone()),
        password: Set(hashed_pass.to_string()),
        ..Default::default()
    }
    .save(conn)
    .await
    .map_err(|err| convert_err_to_500(err, Some("Database error")))?;

    send_verification_mail(
        &user.email,
        &data.activators_reg,
        VerificationType::Register,
    )
    .await
}

#[get("/activate/{token}")]
async fn activate_account(
    token: Path<String>,
    data: web::Data<AppState>,
) -> Result<String, ServiceError> {
    let tokens = data.activators_reg.read().await;
    let Some(email) = tokens.get(&token.into_inner()) else {return Err(ServiceError::BadRequest("Invalid activation link".into()))};
    let conn = &data.conn;
    let user_query = User::find()
        .filter(user::Column::Email.eq(email))
        .one(conn)
        .await
        .map_err(|err| convert_err_to_500(err, Some("Database error")))?;

    let Some(user) = user_query else {return Err(ServiceError::InternalError)};
    let mut user: user::ActiveModel = user.into();
    user.verified = Set(true as i8);

    user.update(conn)
        .await
        .map_err(|err| convert_err_to_500(err, Some("Database error")))?;

    Ok("account verified successfully".to_string())
}

async fn send_verification_mail(
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
