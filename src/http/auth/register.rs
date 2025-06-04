use std::collections::HashMap;

use actix_web::cookie::{time, Cookie, SameSite};
use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse, Responder, post, web};
use chrono::Utc;
use serde::Deserialize;
use serde_json::to_string;

use crate::services::auth::{add_user_to_db, authenticate_user};
use crate::services::csrf::verify_csrf_token;
use crate::services::jwt::generate_jwt_tokens_for_user;
use crate::db::operations::PGPool;
use diesel::result::DatabaseErrorKind as DieselDbError;
use diesel::result::Error as DieselError;
use crate::services::validate::{validate_email, validate_password, validate_phone_number, validate_username};

#[derive(Deserialize)]
struct RegisterForm {
    username: String,
    email: String,
    phone_number: String,
    password: String,
}

#[post("/register")]
pub async fn post_register(
    pool: web::Data<PGPool>,
    req_body: web::Json<RegisterForm>,
    req: HttpRequest,
) -> impl Responder {
    println!(
        "{:?}: Register request from {:?}",
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
    let email = &req_body.email;
    let phone_number = &req_body.phone_number;
    let password = &req_body.password;

    if !validate_username(username.to_string()) {
        return HttpResponse::BadRequest()
            .content_type(ContentType::json())
            .body(r#"{"detail":"invalid username format"}"#);
    }

    if !validate_email(email.to_string()) {
        return HttpResponse::BadRequest()
            .content_type(ContentType::json())
            .body(r#"{"detail":"invalid email format"}"#);
    }

    if !validate_phone_number(phone_number.to_string()) {
        return HttpResponse::BadRequest()
            .content_type(ContentType::json())
            .body(r#"{"detail":"invalid phone number format"}"#);
    }

    if !validate_password(password.to_string()) {
        return HttpResponse::BadRequest()
            .content_type(ContentType::json())
            .body(r#"{"detail":"invalid password format"}"#);
    }

    // create a hash of user password
    let password_hash = match bcrypt::hash(password, 10) {
        Ok(password_hash) => password_hash,
        Err(e) => {
            eprintln!("Internal server error: {}", e);
            return HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(r#"{"detail":"an unexpected error occurred"}"#);
        }
    };

    // attempt to insert a new user into db
    match add_user_to_db(pool.clone(), username, email, phone_number, &password_hash).await {
        Ok(_) => {
            match authenticate_user(pool, username, password).await {
                Ok(user) => {
                    let mut map = HashMap::new();
                    map.insert("id", user.id.to_string());
                    map.insert("username", user.username);
                    map.insert("email", user.email);
                    map.insert("phone_number", user.phone_number);
                    map.insert("two_factor_auth", user.two_factor_auth.to_string());
                    map.insert("profile_pic", user.profile_pic.unwrap_or("".to_string()));
                    map.insert("bio", user.bio.unwrap_or("".to_string()));
                    map.insert("created_at", user.created_at.to_string());

                    let json_str = to_string(&map).unwrap();

                    let (access_token, refresh_token) = generate_jwt_tokens_for_user(user.id.to_string());

                    let access_cookie = Cookie::build("access_token", access_token)
                        .secure(false) // Use `true` in production
                        .http_only(true)
                        .max_age(time::Duration::minutes(15))
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
                        .body(r#"{"detail":"login failed"}"#)
                }
            }
        }
        Err(DieselError::DatabaseError(DieselDbError::UniqueViolation, _)) => {
            HttpResponse::Conflict()
                .content_type(ContentType::json())
                .body(r#"{"detail":"an account with this email or username already exists"}"#)
        }
        Err(e) => {
            eprintln!("Internal server error: {}", e);
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(r#"{"detail":"an unexpected error occurred"}"#)
        }
    }
}
