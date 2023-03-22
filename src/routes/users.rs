use actix_web::{get, post, web, Responder, HttpResponse, FromRequest};
use chrono::Utc;
use jsonwebtoken::{encode, Header, EncodingKey, DecodingKey, decode, Validation};
use sea_orm::{Set, ActiveModelTrait, EntityTrait, ColumnTrait, QueryFilter};

use bcrypt::{hash_with_salt, DEFAULT_COST, verify};
use nanoid::nanoid;

use entity::{user};
use entity::prelude::{User};
use serde::{Serialize, Deserialize};

use crate::appstate::AppState;

const JWT_SECRET: &[u8] = "secret".as_bytes();

struct AuthUser {
    id: i32,
}
use actix_web::http::header;

impl FromRequest for AuthUser{
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let auth_header = req.headers().get(header::AUTHORIZATION)
        .unwrap()
        .to_str()
        .unwrap();

        let token = auth_header.split("Bearer ").collect::<Vec<&str>>()[1];
        let user_id = decode_jwt_token(token.to_string());
        std::future::ready(Ok(AuthUser { id: user_id }))
    }
}
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn create_jwt(uid: i32) -> String {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(60))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: uid.to_string(),
        exp: expiration as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET)).unwrap()
}

fn decode_jwt_token(token: String) -> i32{
    let decoded = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default()
    ).unwrap();

    return decoded.claims.sub.parse::<i32>().unwrap();
}

#[get("/get-user-data")]
async fn get_user_data(user: AuthUser, data: web::Data<AppState>) -> impl Responder {
    let conn = &data.conn;

    let user_query = User::find()
    .filter(user::Column::Id.eq(user.id))
    .one(conn)
    .await;

    if let Err(error) = user_query {
        eprintln!("Database error: {}", error);
        return HttpResponse::InternalServerError().body("Internal server error");
    }

    let user_query = user_query.unwrap();

    if user_query.is_none() {
        return HttpResponse::BadRequest().body("Account does not exist");
    }

    let user = user_query.unwrap();

    HttpResponse::Ok().body(format!("User data: {}", user.username))
}

#[post("/login")]
async fn login(user: web::Json<user::Model>, data: web::Data<AppState>) -> impl Responder {
    let conn = &data.conn;
    let user = user.into_inner();

    let user_query = User::find()
    .filter(user::Column::Username.eq(user.username))
    .one(conn)
    .await;

    if let Err(error) = user_query {
        eprintln!("Database error: {}", error);
        return HttpResponse::InternalServerError().body("Internal server error");
    }

    let user_query = user_query.unwrap();

    if user_query.is_none() {
        return HttpResponse::BadRequest().body("Account does not exist");
    }
    let user_query = user_query.unwrap();
    let result = verify(&user.password, &user_query.password).unwrap();

    if result {
        let token = create_jwt(user_query.id);
        HttpResponse::Ok().body(token)
    } else {
        HttpResponse::Unauthorized().body("Invalid credentials")
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
        return HttpResponse::InternalServerError().body("Internal server error");
    }

    let user_query = user_query.unwrap();
    if user_query.is_some() {
        return HttpResponse::BadRequest().body("Account already exists");
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
        return HttpResponse::InternalServerError().body("Internal server error");
    }

    HttpResponse::Ok().body("Registered successfully")
}
