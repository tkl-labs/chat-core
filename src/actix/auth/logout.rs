use actix_web::{HttpRequest, HttpResponse, Responder, post};
use chrono::Utc;

use crate::actix::auth::jwt::encode_jwt_token;
use actix_web::cookie::{Cookie, SameSite, time};
use actix_web::http::header::ContentType;

#[post("/logout")]
pub async fn post_logout(req: HttpRequest) -> impl Responder {
    let jwt_token = encode_jwt_token("".to_string());
    let mut jwt_cookie = Cookie::build("jwt_token", "")
        .secure(false) // re-enable for HTTPS
        .http_only(true)
        .max_age(time::Duration::minutes(15))
        .finish();

    match jwt_token {
        Ok(val) => {
            jwt_cookie = Cookie::build("jwt_token", val)
                .secure(false) // for localhost, enable secure for HTTPS in prod
                .http_only(true)
                .max_age(time::Duration::minutes(0))
                .same_site(SameSite::Lax)
                .path("/")
                .domain("127.0.0.1")
                .finish()
        }
        Err(_) => {}
    }

    println!(
        "{:?}: Logout request from {:?}",
        Utc::now().timestamp() as usize,
        req.peer_addr()
    );

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .cookie(jwt_cookie)
        .body(r#"{"detail":"logout successful"}"#)
}
