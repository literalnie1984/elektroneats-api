use actix_web::{get, post, web, Responder, HttpResponse, FromRequest};
use sea_orm::{Set, ActiveModelTrait, EntityTrait, ColumnTrait, QueryFilter};

use bcrypt::{hash_with_salt, DEFAULT_COST, verify};
use nanoid::nanoid;

use entity::{user};
use entity::prelude::{User};

use crate::appstate::AppState;

struct AuthUser {
    id: u32,
}

impl FromRequest for AuthUser{
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let user_id = req.headers().get("user_id").unwrap().to_str().unwrap().parse::<u32>().unwrap();
        std::future::ready(Ok(AuthUser { id: user_id }))
    }
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

    let result = verify(&user.password, &user_query.unwrap().password).unwrap();

    if result {
        HttpResponse::Ok().body("Good credentials")
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
