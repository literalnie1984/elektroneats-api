use actix_web::FromRequest;
use serde::{Serialize, Deserialize};
use jsonwebtoken::errors::Error as JwtError;
use chrono::Utc;
use jsonwebtoken::{encode, Header, EncodingKey, DecodingKey, decode, Validation};

const JWT_SECRET: &[u8] = "gioryegioergb389458y85w4huuhierghlgrezhlgh89y5w48954w4w5huoiyh".as_bytes();

pub struct AuthUser {
    pub id: i32,
}
use actix_web::http::header;

impl FromRequest for AuthUser{
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let auth_header = 
        match req.headers().get(header::AUTHORIZATION) {
            Some(l) => l,
            None => return std::future::ready(Err(actix_web::error::ErrorUnauthorized("Missing authorization header"))),
        };

        let auth_header = 
        match auth_header.to_str(){
            Ok(l) => l,
            Err(_) => return std::future::ready(Err(actix_web::error::ErrorUnauthorized("Invalid authorization header"))),
        };

        let token = 
        match auth_header.split("Bearer ").nth(1){
            Some(l) => l,
            None => return std::future::ready(Err(actix_web::error::ErrorUnauthorized("Invalid authorization header"))),
        };


        let user_id = 
        match decode_jwt_token(token.to_string()){
            Ok(l) => l,
            Err(_) => return std::future::ready(Err(actix_web::error::ErrorUnauthorized("Invalid token"))),
        };

        std::future::ready(Ok(AuthUser { id: user_id }))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn create_jwt(uid: i32) -> Result<String, JwtError> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(60))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: uid.to_string(),
        exp: expiration as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET))
}

fn decode_jwt_token(token: String) -> Result<i32, JwtError>{
    let decoded = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default()
    )?;

    return Ok(decoded.claims.sub.parse::<i32>().unwrap());
}