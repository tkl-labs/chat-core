use actix_web::{HttpRequest, HttpResponse, Responder, post};
use actix_web::cookie::{Cookie, SameSite, time};
use actix_web::http::header::ContentType;
use chrono::Utc;

use shared::jwt::{JwtTokenKind, encode_jwt_token};

#[post("/logout")]
pub async fn post_logout(req: HttpRequest) -> impl Responder {
    let access_token = encode_jwt_token("".to_string(), JwtTokenKind::ACCESS);
    let refresh_token = encode_jwt_token("".to_string(), JwtTokenKind::REFRESH);

    match (access_token, refresh_token) {
        (Ok(access_val), Ok(refresh_val)) => {
            let access_cookie = Cookie::build("access_token", access_val)
                .secure(false) // for localhost, enable secure for HTTPS in prod
                .http_only(true)
                .max_age(time::Duration::minutes(0))
                .same_site(SameSite::Lax)
                .path("/")
                .domain("127.0.0.1")
                .finish();

            let refresh_cookie = Cookie::build("refresh_token", refresh_val)
                .secure(false) // for localhost, enable secure for HTTPS in prod
                .http_only(true)
                .max_age(time::Duration::minutes(0))
                .same_site(SameSite::Lax)
                .path("/")
                .domain("127.0.0.1")
                .finish();

            println!(
                "{:?}: POST /auth/logout from {:?}",
                Utc::now().timestamp() as usize,
                req.peer_addr()
            );

            HttpResponse::Ok()
                .content_type(ContentType::json())
                .cookie(access_cookie)
                .cookie(refresh_cookie)
                .body(r#"{"detail":"logout successful"}"#)
        }
        _ => HttpResponse::InternalServerError()
            .content_type(ContentType::json())
            .body(r#"{"detail":"failed to remove tokens"}"#),
    }
}
