use actix_web::{error, get, post, web, HttpResponse, Responder};
use dotenvy::dotenv;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::transport::smtp::PoolConfig;
use lettre::{Message, SmtpTransport, Transport};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use bcrypt::{hash_with_salt, verify, DEFAULT_COST};
use nanoid::nanoid;

use entity::prelude::User;
use entity::user;

use crate::appstate::AppState;

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

//TODO: actually make this function async
#[get("mail_test")]
async fn send_verification_mail(/* conn: &DatabaseConnection */) -> impl Responder {
    let from = "Kantyna-App <kantyna.noreply@mikut.dev>".parse();
    let to = "<piotrekjakobczyk1@gmail.com>".parse();

    if from.is_err() || to.is_err() {
        return HttpResponse::InternalServerError().body("Internal Server Error");
    }

    let mail = Message::builder()
        .from(from.unwrap())
        .to(to.unwrap())
        .subject("Twój kod do kantyny")
        .body(String::from("test email"));

    if mail.is_err() {
        return HttpResponse::InternalServerError().body("Internal Server Error");
    }
    let mail = &mail.unwrap();

    let smtp = SmtpTransport::starttls_relay("mikut.dev")
        .unwrap()
        .credentials(Credentials::new(
            "kantyna.noreply@mikut.dev".to_owned(),
            dotenvy::var("EMAIL_PASS")
                .expect("NO EMAIL_PASS val provided in .evn")
                .to_string(),
        ))
        .authentication(vec![Mechanism::Plain])
        .pool_config(PoolConfig::new().max_size(20))
        .build();

    HttpResponse::Ok().body("Ok")
}
