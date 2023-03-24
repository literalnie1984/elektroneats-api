use actix_web::web::Path;
use actix_web::{get, post, web, Responder};
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::transport::smtp::PoolConfig;
use lettre::{Message, SmtpTransport, Transport};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use bcrypt::{hash_with_salt, verify, DEFAULT_COST};
use nanoid::nanoid;

use entity::prelude::User;
use entity::user;

use crate::appstate::{ActivatorsVec, AppState};

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
        return Err(ServiceError::BadRequest(
            "Account does not exist".to_string(),
        ));
    }

    let user = user_query.unwrap();

    Ok(format!("User data: {}", user.username))
}

#[post("/login")]
async fn login(
    user: web::Json<user::Model>,
    data: web::Data<AppState>,
) -> Result<String, ServiceError> {
    let conn = &data.conn;
    let user = user.into_inner();
    let user_query = User::find()
        .filter(user::Column::Email.eq(user.email))
        .one(conn)
        .await;

    if let Err(error) = user_query {
        eprintln!("Database error: {}", error);
        return Err(ServiceError::InternalError);
    }

    let user_query = user_query.unwrap();

    if user_query.is_none() {
        return Err(ServiceError::BadRequest(
            "Account does not exist".to_string(),
        ));
    }
    let user_query = user_query.unwrap();
    let result = verify(&user.password, &user_query.password).unwrap();

    if result {
        let token = match create_jwt(user_query.id) {
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
async fn register(user: web::Json<user::Model>, data: web::Data<AppState>) -> impl Responder {
    let conn = &data.conn;

    let user = user.into_inner();

    let user_query = User::find()
        .filter(user::Column::Email.eq(&user.email))
        .one(conn)
        .await;

    if let Err(error) = user_query {
        eprintln!("Database error: {}", error);
        return Err(ServiceError::InternalError);
    }

    let user_query = user_query.unwrap();
    if user_query.is_some() {
        return Err(ServiceError::BadRequest(
            "Account already exists".to_string(),
        ));
    }

    let salt = nanoid!(16);
    let salt_copy: [u8; 16] = salt.as_bytes().try_into().unwrap();
    let hashed_pass = hash_with_salt(user.password.as_bytes(), DEFAULT_COST, salt_copy).unwrap();

    let result = user::ActiveModel {
        username: Set(user.username),
        email: Set(user.email.clone()),
        password: Set(hashed_pass.to_string()),
        ..Default::default()
    }
    .save(conn)
    .await;

    if let Err(error) = result {
        eprintln!("Database error: {}", error);
        return Err(ServiceError::InternalError);
    }

    send_verification_mail(&user.email, &data.activators).await
}

//definitely refactor this
#[get("/activate/{token}")]
async fn activate_account(
    token: Path<String>,
    data: web::Data<AppState>,
) -> Result<String, ServiceError> {
    let tokens = data.activators.read().await;
    if let Some(email) = tokens.get(&token.into_inner()) {
        let conn = &data.conn;
        let user_query = User::find()
            .filter(user::Column::Email.eq(email))
            .one(conn)
            .await;

        if let Err(error) = user_query {
            eprintln!("Database error: {}", error);
            return Err(ServiceError::InternalError);
        }

        if let Some(user) = user_query.unwrap() {
            let mut user: user::ActiveModel = user.into();
            user.verified = Set(true as i8);

            if let Err(err) = user.update(conn).await {
                eprintln!("Database error: {}", err);
                return Err(ServiceError::InternalError);
            }

            Ok("account verified successfully".to_string())
        } else {
            return Err(ServiceError::InternalError);
        }
    } else {
        return Err(ServiceError::InternalError);
    }
}

//TODO: actually make this function async
async fn send_verification_mail(
    email: &str,
    activators: &ActivatorsVec,
) -> Result<String, ServiceError> {
    let from = "Kantyna-App <kantyna.noreply@mikut.dev>".parse();
    let to = email.parse();

    if from.is_err() || to.is_err() {
        return Err(ServiceError::InternalError);
    }

    //add email - activation_link combo to current app state
    let mut activators = activators.write().await;
    let activation_link = nanoid!();
    (*activators).insert(activation_link.clone(), email.into());

    let mail = Message::builder()
        .from(from.unwrap())
        .to(to.unwrap())
        .subject("Twój kod do kantyny")
        .body(format!(
            "http://127.0.0.1:4765/api/user/activate/{}",
            activation_link
        ));

    if mail.is_err() {
        return Err(ServiceError::InternalError);
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

    let send_status = smtp.send(mail);

    if send_status.is_err() {
        return Err(ServiceError::InternalError);
    } else {
        Ok("Registered successfully; email send".to_string())
    }
}
