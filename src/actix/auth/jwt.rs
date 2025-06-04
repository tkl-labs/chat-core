use chrono::{Duration, Utc};
use dotenv::dotenv;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::env;

use crate::actix::auth::me::get_user_by_id;
use crate::database::init::PGPool;
use actix_web::cookie::{Cookie, SameSite, time};
use actix_web::http::header::ContentType;
use actix_web::web;
use actix_web::{HttpRequest, HttpResponse, Responder, post};

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

pub fn decode_jwt_token(
    token: String,
    token_kind: JwtTokenKind,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    dotenv().ok();

    let jwt_secret = match token_kind {
        JwtTokenKind::ACCESS => env::var("JWT_ACCESS_TOKEN_SECRET")
            .expect("ERROR: JWT_ACCESS_TOKEN_SECRET must be present in '.env'"),
        JwtTokenKind::REFRESH => env::var("JWT_REFRESH_TOKEN_SECRET")
            .expect("ERROR: JWT_REFRESH_TOKEN_SECRET must be present in '.env'"),
    };

    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

#[post("/refresh")]
pub async fn post_refresh(pool: web::Data<PGPool>, req: HttpRequest) -> impl Responder {
    println!(
        "{:?}: Token refresh request from {:?}",
        Utc::now().timestamp() as usize,
        req.peer_addr()
    );

    // extract refresh token from cookie
    let refresh_token = match req.cookie("refresh_token") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return HttpResponse::Forbidden()
                .content_type(ContentType::json())
                .body(r#"{"detail":"missing refresh token"}"#);
        }
    };

    // decode and validate refresh token
    let claim = match decode_jwt_token(refresh_token, JwtTokenKind::REFRESH) {
        Ok(claim) => claim,
        Err(_) => {
            return HttpResponse::Forbidden()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid refresh token"}"#);
        }
    };

    let user_id = claim.sub;

    match get_user_by_id(pool, &user_id).await {
        Ok(_) => {
            // let origin: String = match req.headers().get("host") {
            //     Some(val) => val.to_str().unwrap_or("127.0.0.1").to_string(),
            //     None => "127.0.0.1".to_string(),
            // };
            
            let new_access_token = match encode_jwt_token(user_id.to_string(), JwtTokenKind::ACCESS)
            {
                Ok(token) => token,
                Err(e) => {
                    eprintln!(
                        "{:?}: Failed to encode access token: {:?}",
                        Utc::now().timestamp() as usize,
                        e
                    );
                    return HttpResponse::InternalServerError()
                        .content_type(ContentType::json())
                        .body(r#"{"detail":"internal server error"}"#);
                }
            };

            let access_cookie = Cookie::build("access_token", &new_access_token)
                .secure(false) // for localhost, enable secure for HTTPS in prod
                .http_only(true)
                .max_age(time::Duration::minutes(15))
                .same_site(SameSite::Lax)
                .path("/")
                //.domain(&origin)
                .finish();

            HttpResponse::Ok()
                .content_type(ContentType::json())
                .cookie(access_cookie)
                .body(r#"{"detail":"access token refreshed successfully"}"#)
        }
        Err(e) => {
            eprintln!(
                "{:?}: User fetching failed: {:?}",
                Utc::now().timestamp() as usize,
                e
            );

            HttpResponse::Forbidden()
                .content_type(ContentType::json())
                .body(r#"{"detail":"User not found"}"#)
        }
    }
}
