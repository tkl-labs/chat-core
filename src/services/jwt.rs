use chrono::{Duration, Utc};
use dotenv::dotenv;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, errors::ErrorKind, decode, encode};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // aud: String, // Optional. Audience
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    iss: String, // Optional. Issuer
    // nbf: usize, // Optional. Not Before (as UTC timestamp)
    pub sub: String, // Optional. Subject (whom token refers to)
}

pub enum JwtTokenKind {
    ACCESS,
    REFRESH,
}
pub enum JwtError {
    Expired,
    Invalid,
    Other(String),
}

fn create_jwt_claims(user_id: String, token_type: JwtTokenKind) -> Claims {
    let now = Utc::now();

    let exp = match token_type {
        JwtTokenKind::ACCESS => (now + Duration::minutes(15)).timestamp() as usize,
        JwtTokenKind::REFRESH => (now + Duration::days(7)).timestamp() as usize,
    };

    let claim = Claims {
        // aud: "http://127.0.0.1:3000",
        exp: exp,
        iat: now.timestamp() as usize,
        iss: "http://127.0.0.1:8080".to_string(),
        // nbf: now,
        sub: user_id,
    };

    claim
}

pub fn encode_jwt_token(
    user_id: String,
    token_kind: JwtTokenKind,
) -> Result<String, jsonwebtoken::errors::Error> {
    dotenv().ok();

    let jwt_secret = match token_kind {
        JwtTokenKind::ACCESS => env::var("JWT_ACCESS_TOKEN_SECRET")
            .expect("ERROR: JWT_ACCESS_TOKEN_SECRET must be present in '.env'"),
        JwtTokenKind::REFRESH => env::var("JWT_REFRESH_TOKEN_SECRET")
            .expect("ERROR: JWT_REFRESH_TOKEN_SECRET must be present in '.env'"),
    };

    let claims = create_jwt_claims(user_id, token_kind);

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
}

pub fn decode_jwt_token(token: String, token_kind: JwtTokenKind) -> Result<Claims, JwtError> {
    dotenv().ok();

    let jwt_secret = match token_kind {
        JwtTokenKind::ACCESS => env::var("JWT_ACCESS_TOKEN_SECRET")
            .expect("ERROR: JWT_ACCESS_TOKEN_SECRET must be present in '.env'"),
        JwtTokenKind::REFRESH => env::var("JWT_REFRESH_TOKEN_SECRET")
            .expect("ERROR: JWT_REFRESH_TOKEN_SECRET must be present in '.env'"),
    };

    let validation = Validation::default();

    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &validation,
    ) {
        Ok(token_data) => Ok(token_data.claims),
        Err(err) => match *err.kind() {
            ErrorKind::ExpiredSignature => Err(JwtError::Expired),
            ErrorKind::InvalidToken => Err(JwtError::Invalid),
            _ => Err(JwtError::Other(err.to_string())),
        },
    }
}

pub fn generate_jwt_tokens_for_user(id: String) -> (String, String) {
    let access_token = match encode_jwt_token(id.clone(), JwtTokenKind::ACCESS) {
        Ok(token) => token,
        Err(_) => return ("".to_string(), "".to_string()),
    };

    let refresh_token = match encode_jwt_token(id, JwtTokenKind::REFRESH) {
        Ok(token) => token,
        Err(_) => return ("".to_string(), "".to_string()),
    };

    (access_token, refresh_token)
}

pub fn clear_jwt_tokens() -> (String, String) {
    ("".to_string(), "".to_string())
}
