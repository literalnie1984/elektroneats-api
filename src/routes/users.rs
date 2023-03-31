use actix_web::web::Path;
use actix_web::{get, post, web, Responder};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, Set};
use stripe::{CreateCustomer, Customer};

use bcrypt::{hash_with_salt, verify, DEFAULT_COST};
use nanoid::nanoid;

use entity::prelude::User;
use entity::user;

use crate::appstate::AppState;
use crate::enums::VerificationType;
use crate::routes::structs::UserJson;
use crate::{map_db_err, send_verification_mail};

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
        .map_err(map_db_err)?;

    let Some(user) = user_query else {return Err(ServiceError::BadRequest("Account does not exist".into()))};

    Ok(web::Json(UserJson {
        username: user.username,
        admin: user.admin != 0,
    }))
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
        .map_err(map_db_err)?;

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
        .map_err(map_db_err)?;

    let Some(user) = user_query else {return Err(ServiceError::InternalError)};

    let res = user.delete(conn).await.map_err(map_db_err)?;

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
        .map_err(map_db_err)?;

    let Some(user_query) = user_query else {return Err(ServiceError::BadRequest("Account does not exist".into()))};
    let result = verify(&user.password, &user_query.password).unwrap();

    if result {
        let token = match create_jwt(user_query.id, user_query.admin) {
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
        .map_err(map_db_err)?;

    let Some(user) = user_query else {return Err(ServiceError::InternalError)};
    let mut user: user::ActiveModel = user.into();
    user.verified = Set(true as i8);

    user.update(conn).await.map_err(map_db_err)?;

    Ok("account verified successfully".to_string())
}

#[post("/add_balance/{amount}")]
async fn add_balance(
    user: AuthUser,
    data: web::Data<AppState>,
    amount: web::Path<u32>,
) -> Result<String, ServiceError> {
    let amount = amount.into_inner();
    let secret_key = dotenvy::var("STRIPE_SECRET").expect("No STRIPE_SECRET variable in dotenv");
    let client = stripe::Client::new(secret_key);
    let email = "test@gmail.com";
    let username = "temp";

    Customer::retrieve(client, id, expand)
    let customer = Customer::create(
        &client,
        CreateCustomer {
            name: Some(username),
            email: Some(email),
            ..Default::default()
        },
    );

    Ok("TODO".into())
}
