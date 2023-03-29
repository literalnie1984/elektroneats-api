use actix_web::http::header;
use actix_web::FromRequest;
use chrono::Utc;
use jsonwebtoken::errors::Error as JwtError;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::errors::ServiceError;

//change @ Release
const JWT_SECRET: &[u8] =
    "gioryegioergb389458y85w4huuhierghlgrezhlgh89y5w48954w4w5huoiyh".as_bytes();

pub struct AuthUser {
    pub id: i32,
    pub is_admin: bool,
}

impl FromRequest for AuthUser {
    type Error = ServiceError;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        fn return_func(err: ServiceError) -> std::future::Ready<Result<AuthUser, ServiceError>> {
            std::future::ready(Err(err))
        }
        let auth_header = match req.headers().get(header::AUTHORIZATION) {
            Some(l) => l,
            None => {
                return return_func(ServiceError::BadRequest(
                    "No authorization header".to_string(),
                ))
            }
        };

        let auth_header = match auth_header.to_str() {
            Ok(l) => l,
            Err(_) => {
                return return_func(ServiceError::BadRequest(
                    "Ivalid authorization header".to_string(),
                ))
            }
        };

        let token = match auth_header.split("Bearer ").nth(1) {
            Some(l) => l,
            None => {
                return return_func(ServiceError::BadRequest(
                    "Ivalid authorization header".to_string(),
                ))
            }
        };

        let user = match decode_jwt_token(token.to_string()) {
            Ok(l) => l,
            Err(_) => return return_func(ServiceError::JWTInvalidToken),
        };

        std::future::ready(Ok(user))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    is_admin: bool,
    exp: usize,
}

pub fn create_jwt(uid: i32, is_admin: i8) -> Result<String, JwtError> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(60))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: uid.to_string(),
        is_admin: is_admin == 1,
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
}

fn decode_jwt_token(token: String) -> Result<AuthUser, ServiceError> {
    let decoded = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )
    .map_err(|_| ServiceError::JWTInvalidToken)?;

    let uid = decoded
        .claims
        .sub
        .parse::<i32>()
        .map_err(|_| ServiceError::JWTInvalidToken)?;
    let is_admin = decoded.claims.is_admin;

    Ok(AuthUser{id: uid, is_admin: is_admin})
}
