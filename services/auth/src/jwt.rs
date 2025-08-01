use actix_web::cookie::{Cookie, SameSite, time};
use actix_web::http::header::ContentType;
use actix_web::web;
use actix_web::{HttpRequest, HttpResponse, Responder, post};
use chrono::Utc;
use opentelemetry::{
    KeyValue, global,
    trace::{Span, Tracer},
};

use shared::database::PGPool;
use shared::jwt::{JwtTokenKind, encode_jwt_token, extract_user_id};
use shared::profile::get_user_by_id;

#[post("/refresh")]
pub async fn post_refresh(pool: web::Data<PGPool>, req: HttpRequest) -> impl Responder {
    let tracer = global::tracer("my_tracer");

    let mut span = tracer.start("post_refresh");
    span.set_attribute(KeyValue::new("rpc.method", "post_refresh"));

    println!(
        "{:?}: POST /auth/refresh from {:?}",
        Utc::now().timestamp() as usize,
        req.peer_addr()
    );

    // extract user id from refresh token
    let user_id = match extract_user_id(&req, JwtTokenKind::REFRESH) {
        Ok(id) => id,
        Err(resp) => {
            span.end();
            return resp;
        }
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

                    span.end();
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

            span.end();
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

            span.end();
            HttpResponse::Forbidden()
                .content_type(ContentType::json())
                .body(r#"{"detail":"User not found"}"#)
        }
    }
}
