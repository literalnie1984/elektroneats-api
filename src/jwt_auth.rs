use actix_web::http::header;
use actix_web::FromRequest;
use chrono::Utc;
use jsonwebtoken::errors::{Error as JwtError, ErrorKind};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::errors::ServiceError;

//change @ Release
const JWT_SECRET: &[u8] =
    "gioryegioergb389458y85w4huuhierghlgrezhlgh89y5w48954w4w5huoiyh".as_bytes();

pub struct AuthUser {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
    pub is_verified: bool,
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
                    "Invalid authorization header".to_string(),
                ))
            }
        };

        let token = match auth_header.split("Bearer ").nth(1) {
            Some(l) => l,
            None => {
                return return_func(ServiceError::BadRequest(
                    "Invalid authorization header".to_string(),
                ))
            }
        };

        let user = match decode_access_token(token.to_string()) {
            Ok(l) => l,
            Err(_) => return return_func(ServiceError::JWTInvalidToken("Access".to_string())),
        };

        std::future::ready(Ok(user))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    sub: String,
    username: String,
    email: String,
    is_admin: bool,
    is_verified: bool,
    exp: usize,
}

impl AccessTokenClaims{
    pub fn new(id: i32, username: &str, email: &str, is_admin: i8, is_verified: bool, exp_seconds: i64) -> Self{
        Self{
            sub: id.to_string(),
            username: username.to_string(),
            email: email.to_string(),
            is_admin: is_admin == 1,
            exp: get_expiration(exp_seconds),
            is_verified: is_verified
        }
    }
}

fn decode_access_token(token: String) -> Result<AuthUser, ServiceError> {
    let decoded = decode::<AccessTokenClaims>(
        &token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )
    .map_err(|err| map_decode_err(err, "Access"))?;

    let uid = decoded
        .claims
        .sub
        .parse::<i32>()
        .map_err(|_| ServiceError::JWTInvalidToken("Access".to_string()))?;

    Ok(AuthUser {
        id: uid,
        is_admin: decoded.claims.is_admin,
        username: decoded.claims.username,
        email: decoded.claims.email,
        is_verified: decoded.claims.is_verified
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    sub: String,
    exp: usize,
}

impl RefreshTokenClaims{
    pub fn new(id: i32, exp_seconds: i64) -> Self{
        Self{
            sub: id.to_string(),
            exp: get_expiration(exp_seconds)
        }
    }
}

pub fn decode_refresh_token(token: &str) -> Result<i32, ServiceError>{
    let decoded = decode::<RefreshTokenClaims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )
    .map_err(|err| map_decode_err(err, "Refresh"))?;

    let uid = decoded
        .claims
        .sub
        .parse::<i32>()
        .map_err(|_| ServiceError::JWTInvalidToken("Refresh".to_string()))?;

    Ok(uid)
}

pub fn get_expiration(seconds: i64) -> usize{
    Utc::now()
        .checked_add_signed(chrono::Duration::seconds(seconds))
        .expect("valid timestamp")
        .timestamp() as usize
}

pub fn encode_jwt<T>(data: &T) -> Result<String, JwtError>
where
    T: Serialize,
{
    encode(
        &Header::default(),
        &data,
        &EncodingKey::from_secret(JWT_SECRET),
    )
}

pub fn map_decode_err(err: JwtError, token_name: &str) -> ServiceError
{
    if let ErrorKind::ExpiredSignature = err.kind(){
        return ServiceError::JWTExpiredToken(token_name.to_string());
    }

    ServiceError::JWTInvalidToken(token_name.to_string())
}
