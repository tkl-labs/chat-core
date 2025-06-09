use std::collections::HashMap;

use actix_web::cookie::{Cookie, SameSite, time};
use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse, Responder, post, web};
use chrono::Utc;
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
    println!(
        "{:?}: POST /auth/login from {:?}",
        Utc::now().timestamp() as usize,
        req.peer_addr()
    );

    let verify_csrf = verify_csrf_token(&req);

    if !verify_csrf {
        return HttpResponse::Unauthorized()
            .content_type(ContentType::json())
            .body(r#"{"detail":"csrf failed"}"#);
    }

    let username = req_body.username.trim();
    let password = &req_body.password;

    if !validate_existing_username(&username) {
        return HttpResponse::Unauthorized()
            .content_type(ContentType::json())
            .body(r#"{"detail":"invalid login"}"#);
    }

    if !validate_password(password.to_string()) {
        return HttpResponse::Unauthorized()
            .content_type(ContentType::json())
            .body(r#"{"detail":"invalid login"}"#);
    }

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

            HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"incorrect login details"}"#)
        }
    }
}
