use std::collections::HashMap;

use actix_web::cookie::{Cookie, SameSite, time};
use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse, Responder, post, web};
use chrono::Utc;
use opentelemetry::trace::TraceContextExt;
use opentelemetry::{
    Context,
    global,
    KeyValue,
    trace::{Span, Tracer},
};
use serde::Deserialize;
use serde_json::to_string;

use crate::auth::authenticate_user;
use shared::csrf::verify_csrf_token;
use shared::database::PGPool;
use shared::jwt::generate_jwt_tokens_for_user;
use shared::validate::{validate_existing_username, validate_password};

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[post("/login")]
pub async fn post_login(
    pool: web::Data<PGPool>,
    req_body: web::Json<LoginForm>,
    req: HttpRequest,
) -> impl Responder {
    let tracer = global::tracer("my_tracer");

    let mut init_span = tracer.start("post_login");
    init_span.set_attribute(KeyValue::new("rpc.method", "post_login"));

    println!(
        "{:?}: POST /auth/login from {:?}",
        Utc::now().timestamp() as usize,
        req.peer_addr()
    );

    let mut csrf_span = tracer.start_with_context("verify_csrf_token", &Context::current().with_span(init_span));
    csrf_span.set_attribute(KeyValue::new("rpc.method", "verify_csrf_token"));
    let verify_csrf = verify_csrf_token(&req);

    if !verify_csrf {
        csrf_span.end();
        return HttpResponse::Unauthorized()
            .content_type(ContentType::json())
            .body(r#"{"detail":"csrf failed"}"#);
    }
    csrf_span.end();

    let username = req_body.username.trim();
    let password = &req_body.password;

    let mut validate_username_span = tracer.start_with_context("validate_existing_username", &Context::current().with_span(csrf_span));
    validate_username_span.set_attribute(KeyValue::new("rpc.method", "validate_existing_username"));
    if !validate_existing_username(&username) {
        validate_username_span.end();
        return HttpResponse::Unauthorized()
            .content_type(ContentType::json())
            .body(r#"{"detail":"invalid login"}"#);
    }
    validate_username_span.end();

    let mut validate_password_span = tracer.start_with_context("validate_existing_password", &Context::current().with_span(validate_username_span));
    validate_password_span.set_attribute(KeyValue::new("rpc.method", "validate_existing_password"));
    if !validate_password(password.to_string()) {
        validate_password_span.end();
        return HttpResponse::Unauthorized()
            .content_type(ContentType::json())
            .body(r#"{"detail":"invalid login"}"#);
    }
    validate_password_span.end();

    let mut auth_span = tracer.start_with_context("authenticate_user", &Context::current().with_span(validate_password_span));
    auth_span.set_attribute(KeyValue::new("rpc.method", "authenticate_user"));
    match authenticate_user(pool, username, password).await {
        Ok(user) => {
            let mut map = HashMap::new();
            map.insert("id", user.id.to_string());
            map.insert("username", user.username);
            map.insert("email", user.email);
            map.insert("phone_number", user.phone_number);
            map.insert("two_factor_auth", user.two_factor_auth.to_string());
            map.insert("profile_pic", user.profile_pic.unwrap_or_default());
            map.insert("bio", user.bio.unwrap_or_default());
            map.insert("created_at", user.created_at.to_string());

            let json_str = to_string(&map).unwrap();

            let (access_token, refresh_token) = generate_jwt_tokens_for_user(user.id.to_string());

            let access_cookie = Cookie::build("access_token", access_token)
                .secure(false) // Use `true` in production
                .http_only(true)
                .max_age(time::Duration::seconds(1))
                .same_site(SameSite::Lax)
                .path("/")
                .domain("127.0.0.1")
                .finish();

            let refresh_cookie = Cookie::build("refresh_token", refresh_token)
                .secure(false)
                .http_only(true)
                .max_age(time::Duration::days(7))
                .same_site(SameSite::Lax)
                .path("/")
                .domain("127.0.0.1")
                .finish();

            auth_span.end();
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .cookie(access_cookie)
                .cookie(refresh_cookie)
                .body(json_str)
        }
        Err(e) => {
            eprintln!(
                "{:?}: Login failed: {:?}",
                Utc::now().timestamp() as usize,
                e
            );

            auth_span.end();
            HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"incorrect login details"}"#)
        }
    }
}
