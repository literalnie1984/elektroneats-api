use std::mem;

use actix_web::web::Path;
use actix_web::{delete, get, post, put, web, Responder};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, Set};

use bcrypt::{hash_with_salt, verify, DEFAULT_COST};
use nanoid::nanoid;

use entity::prelude::User;
use entity::user;
use serde::Deserialize;

use crate::appstate::AppState;
use crate::enums::VerificationType;
use crate::routes::structs::{RefreshTokenRequest, TokenGenResponse, UserJson};
use crate::{convert_err_to_500, map_db_err, send_verification_mail};

use crate::errors::ServiceError;
use crate::jwt_auth::AuthUser;
use crate::jwt_auth::{decode_refresh_token, encode_jwt, AccessTokenClaims, RefreshTokenClaims};
use crate::routes::structs::{UserChangePassword, UserLogin, UserRegister};

use log::error;

#[put("/password")]
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
        .map_err(map_db_err)?;

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
            Err(ServiceError::InternalError)
        }
    }
}

#[get("/data")]
async fn get_user_data(user: AuthUser) -> impl Responder {
    // let conn = &data.conn;

    // let user_query = User::find()
    //     .filter(user::Column::Id.eq(user.id))
    //     .one(conn)
    //     .await
    //     .map_err(map_db_err)?;

    // let Some(user) = user_query else {return Err(ServiceError::BadRequest("Account does not exist".into()))};

    web::Json(UserJson {
        username: user.username,
        admin: user.is_admin,
    })
}

#[post("/delete")]
async fn get_delete_mail(
    user: AuthUser,
    data: web::Data<AppState>,
) -> Result<impl Responder, ServiceError> {
    let conn = &data.conn;

    let user_query = User::find()
        .filter(user::Column::Id.eq(user.id))
        .one(conn)
        .await
        .map_err(map_db_err)?;

    let Some(user) = user_query else {return Err(ServiceError::BadRequest("Account does not exist".into()))};

    send_verification_mail(&user.email, &data.activators_del, VerificationType::Delete).await
}

#[delete("/delete/{token}")]
async fn delete_acc(
    user: AuthUser,
    data: web::Data<AppState>,
    token: Path<String>,
) -> Result<impl Responder, ServiceError> {
    let tokens = data.activators_del.read().await;
    let token = &token.into_inner();
    let Some(email) = tokens.get(token) else {return Err(ServiceError::BadRequest("Invalid deletion token!".into()))};

    if user.email != *email {
        return Err(ServiceError::BadRequest("Invalid deletion code!".into()));
    }

    let conn = &data.conn;
    let user_query: Option<user::Model> = User::find()
        .filter(user::Column::Email.eq(user.email))
        .one(conn)
        .await
        .map_err(map_db_err)?;

    let Some(user) = user_query else {return Err(ServiceError::InternalError)};

    let res = user.delete(conn).await.map_err(map_db_err)?;

    if res.rows_affected == 1 {
        Ok("Deleted account successfully")
    } else {
        Err(ServiceError::InternalError)
    }
}

#[post("/refresh-token")]
async fn refresh_token(
    mut refresh_token: web::Json<RefreshTokenRequest>,
    data: web::Data<AppState>,
) -> Result<web::Json<TokenGenResponse>, ServiceError> {
    let conn = &data.conn;
    let uid = decode_refresh_token(&refresh_token.refresh_token)?;

    let user_query = User::find()
        .filter(user::Column::Id.eq(uid))
        .one(conn)
        .await
        .map_err(map_db_err)?;
    let Some(user_query) = user_query else {return Err(ServiceError::BadRequest("Account does not exist".into()))};

    let new_access_token = encode_jwt(&AccessTokenClaims::new(
        user_query.id,
        &user_query.username,
        &user_query.email,
        user_query.admin,
        user_query.verified == 1,
        60 * 10,
    ))
    .map_err(|err| convert_err_to_500(err, Some("Error creating new access token")))?;

    Ok(web::Json(TokenGenResponse {
        access_token: new_access_token,
        refresh_token: mem::take(&mut refresh_token.refresh_token),
    }))
}

#[post("/login")]
async fn login(
    user: web::Json<UserLogin>,
    data: web::Data<AppState>,
) -> Result<web::Json<TokenGenResponse>, ServiceError> {
    let conn = &data.conn;
    let user = user.into_inner();
    let user_query = User::find()
        .filter(user::Column::Email.eq(user.email))
        .one(conn)
        .await
        .map_err(map_db_err)?;

    let Some(user_query) = user_query else {return Err(ServiceError::BadRequest("Account does not exist".into()))};
    let result = verify(&user.password, &user_query.password).unwrap();

    if !result {
        return Err(ServiceError::Unauthorized(
            "Invalid credentials".to_string(),
        ));
    }

    let access_token = encode_jwt(&AccessTokenClaims::new(
        user_query.id,
        &user_query.username,
        &user_query.email,
        user_query.admin,
        user_query.verified == 1,
        60 * 10,
    ))
    .map_err(|err| convert_err_to_500(err, Some("Error creating access token")))?;

    let ref_token = encode_jwt(&RefreshTokenClaims::new(user_query.id, 60 * 60 * 24))
        .map_err(|err| convert_err_to_500(err, Some("Error creating refresh token")))?;

    Ok(web::Json(TokenGenResponse {
        access_token,
        refresh_token: ref_token,
    }))
}

#[post("/register")]
async fn register(user: web::Json<UserRegister>, data: web::Data<AppState>) -> impl Responder {
    let conn = &data.conn;

    let user = user.into_inner();

    let user_query = User::find()
        .filter(user::Column::Email.eq(&user.email))
        .one(conn)
        .await
        .map_err(map_db_err)?;

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
    .map_err(map_db_err)?;

    send_verification_mail(
        &user.email,
        &data.activators_reg,
        VerificationType::Register,
    )
    .await
}

#[derive(Deserialize)]
struct Email {
    email: String,
}

#[post("/resend-activation")]
async fn resend_activation(
    data: web::Data<AppState>,
    email: web::Json<Email>,
) -> Result<String, ServiceError> {
    send_verification_mail(
        &email.email,
        &data.activators_reg,
        VerificationType::Register,
    )
    .await
}

#[post("/activate/{token}")]
async fn activate_account(
    token: Path<String>,
    data: web::Data<AppState>,
    mut email: web::Json<Email>,
) -> Result<String, ServiceError> {
    let tokens = data.activators_reg.read().await;
    let token = &token.into_inner();
    let email = mem::take(&mut email.email);
    let Some(email2) = tokens.get(token) else {return Err(ServiceError::BadRequest("Invalid activation link".into()))};
    if email != *email2 {
        return Err(ServiceError::BadRequest("Bad activation code".into()));
    }
    let conn = &data.conn;
    let user_query = User::find()
        .filter(user::Column::Email.eq(email))
        .one(conn)
        .await
        .map_err(map_db_err)?;

    let Some(user) = user_query else {return Err(ServiceError::InternalError)};
    let mut user: user::ActiveModel = user.into();
    user.verified = Set(true as i8);

    user.update(conn).await.map_err(map_db_err)?;

    Ok("account verified successfully".to_string())
}
