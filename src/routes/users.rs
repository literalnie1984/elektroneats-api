use actix_web::{get, post, web, Responder, HttpResponse};

use sea_orm::{Set, ActiveModelTrait, EntityTrait, ColumnTrait, QueryFilter};

use bcrypt::{hash_with_salt, DEFAULT_COST, verify};
use nanoid::nanoid;

use entity::{user};
use entity::prelude::{User};

use crate::appstate::AppState;

use crate::errors::ServiceError;
use crate::jwt_auth::create_jwt;
use crate::jwt_auth::AuthUser;

#[get("/get-user-data")]
async fn get_user_data(user: AuthUser, data: web::Data<AppState>) -> impl Responder {
    let conn = &data.conn;

    let user_query = User::find()
    .filter(user::Column::Id.eq(user.id))
    .one(conn)
    .await;

    if let Err(error) = user_query {
        eprintln!("Database error: {}", error);
        return Err(ServiceError::InternalError);
    }

    let user_query = user_query.unwrap();

    if user_query.is_none() {
        return Err(ServiceError::BadRequest("Account does not exist".to_string()));
    }

    let user = user_query.unwrap();

    Ok(format!("User data: {}", user.username))
}

#[post("/login")]
async fn login(user: web::Json<user::Model>, data: web::Data<AppState>) -> Result<String, ServiceError> {
    let conn = &data.conn;
    let user = user.into_inner();

    let user_query = User::find()
    .filter(user::Column::Username.eq(user.username))
    .one(conn)
    .await;

    if let Err(error) = user_query {
        eprintln!("Database error: {}", error);
        return Err(ServiceError::InternalError);
    }

    let user_query = user_query.unwrap();

    if user_query.is_none() {
        return Err(ServiceError::BadRequest("Account does not exist".to_string()));
    }
    let user_query = user_query.unwrap();
    let result = verify(&user.password, &user_query.password).unwrap();

    if result {
        let token = 
        match create_jwt(user_query.id){
            Ok(token) => token,
            Err(error) => {
                eprintln!("Error creating token: {}", error);
                return Err(ServiceError::InternalError);
            }
        };

        Ok(token)
    } else {
        Err(ServiceError::Unauthorized("Invalid credentials".to_string()))
    }
}

#[post("/register")]
async fn register(user: web::Json<user::Model>, data: web::Data<AppState>) -> impl Responder {
    let conn = &data.conn;

    let user = user.into_inner();

    let user_query = User::find()
    .filter(user::Column::Username.eq(&user.username))
    .one(conn)
    .await;

    if let Err(error) = user_query {
        eprintln!("Database error: {}", error);
        return Err(ServiceError::InternalError);
    }

    let user_query = user_query.unwrap();
    if user_query.is_some() {
        return Err(ServiceError::BadRequest("Account already exists".to_string()));
    }

    let salt = nanoid!(16);
    let salt_copy: [u8; 16] = salt.as_bytes().try_into().unwrap();
    let hashed_pass = hash_with_salt(user.password.as_bytes(), DEFAULT_COST, salt_copy).unwrap();

    let result = user::ActiveModel {
        username: Set(user.username),
        password: Set(hashed_pass.to_string()),
        ..Default::default()
    }
    .save(conn)
    .await;

    if let Err(error) = result {
        eprintln!("Database error: {}", error);
        return Err(ServiceError::InternalError);
    }

    Ok("Account created successfully".to_string())
}
