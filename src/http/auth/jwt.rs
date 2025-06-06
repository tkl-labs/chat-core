use actix_web::cookie::{Cookie, SameSite, time};
use actix_web::http::header::ContentType;
use actix_web::web;
use actix_web::{HttpRequest, HttpResponse, Responder, post};
use chrono::Utc;

use crate::db::operations::PGPool;
use crate::services::jwt::{JwtTokenKind, encode_jwt_token, extract_user_id};
use crate::services::profile::get_user_by_id;

#[post("/refresh")]
pub async fn post_refresh(pool: web::Data<PGPool>, req: HttpRequest) -> impl Responder {
    println!(
        "{:?}: Token refresh request from {:?}",
        Utc::now().timestamp() as usize,
        req.peer_addr()
    );

    // extract user id from access token
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

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
