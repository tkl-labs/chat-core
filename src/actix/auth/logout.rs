use actix_web::{HttpRequest, HttpResponse, Responder, post};
use chrono::Utc;

use crate::actix::auth::jwt::encode_jwt_token;
use actix_web::cookie::{Cookie, SameSite, time};
use actix_web::http::header::ContentType;

#[post("/logout")]
pub async fn post_logout(req: HttpRequest) -> impl Responder {
    let access_token = encode_jwt_token("".to_string(), "access".to_string());
    let refresh_token = encode_jwt_token("".to_string(), "refresh".to_string());

    let access_cookie;
    let refresh_cookie;

    match (access_token, refresh_token) {
        (Ok(access_val), Ok(refresh_val)) => {
            access_cookie = Cookie::build("access_token", access_val)
                .secure(false) // for localhost, enable secure for HTTPS in prod
                .http_only(true)
                .max_age(time::Duration::minutes(0))
                .same_site(SameSite::Lax)
                .path("/")
                .domain("127.0.0.1")
                .finish();

            refresh_cookie = Cookie::build("refresh_token", refresh_val)
                .secure(false) // for localhost, enable secure for HTTPS in prod
                .http_only(true)
                .max_age(time::Duration::minutes(0))
                .same_site(SameSite::Lax)
                .path("/")
                .domain("127.0.0.1")
                .finish();
        }
        _ => {
            return HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(r#"{"detail":"failed to remove tokens"}"#);
        }
    }

    println!(
        "{:?}: Logout request from {:?}",
        Utc::now().timestamp() as usize,
        req.peer_addr()
    );

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .body(r#"{"detail":"logout successful"}"#)
}
