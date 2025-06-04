use chrono::Utc;

use crate::services::profile::get_user_by_id;
use crate::db::operations::PGPool;
use crate::services::jwt::{decode_jwt_token, encode_jwt_token, JwtTokenKind};
use actix_web::cookie::{Cookie, SameSite, time};
use actix_web::http::header::ContentType;
use actix_web::web;
use actix_web::{HttpRequest, HttpResponse, Responder, post};

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
                .domain("127.0.0.1")
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