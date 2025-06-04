use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse, Responder, get, patch, web};
use chrono::Utc;
use uuid::Uuid;

use crate::services::profile::{apply_profile_update, get_user_by_id};
use crate::services::jwt::{JwtTokenKind, decode_jwt_token};
use crate::db::operations::PGPool;
use crate::models::UpdateUser;
use crate::services::validate::{validate_bio, validate_email, validate_phone_number, validate_profile_pic, validate_username};

use std::collections::HashMap;
use serde_json::to_string;

#[get("/me")]
pub async fn get_me(pool: web::Data<PGPool>, req: HttpRequest) -> impl Responder {
    println!(
        "{:?}: Me request from {:?}",
        Utc::now().timestamp() as usize,
        req.peer_addr()
    );

    // extract access token from cookie
    let access_token = match req.cookie("access_token") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            eprintln!(
                "{:?}: extracting failed:",
                Utc::now().timestamp() as usize,
            );
            return HttpResponse::Forbidden()
                .content_type(ContentType::json())
                .body(r#"{"detail":"missing jwt token"}"#);
        }
    };

    // decode and validate JWT token
    let claim = match decode_jwt_token(access_token, JwtTokenKind::ACCESS) {
        Ok(claim) => claim,
        Err(_) => {
            eprintln!(
                "{:?}: decoding failed:",
                Utc::now().timestamp() as usize,
            );
            return HttpResponse::Forbidden()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid access token"}"#);
        }
    };

    let user_id = claim.sub;

    match get_user_by_id(pool, &user_id).await {
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

            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(json_str)
        }
        Err(e) => {
            eprintln!(
                "{:?}: User fetching failed: {:?}",
                Utc::now().timestamp() as usize,
                e
            );

            HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"User not found"}"#)
        }
    }
}

#[get("/profile")]
pub async fn get_profile(
    pool: web::Data<PGPool>,
    req: HttpRequest,
) -> impl Responder {
    println!(
        "{:?}: Get profile request from {:?}",
        Utc::now().timestamp() as usize,
        req.peer_addr()
    );

    // extract access token from cookie
    let access_token = match req.cookie("access_token") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"missing jwt token"}"#);
        }
    };

    // decode and validate JWT token
    let claim = match decode_jwt_token(access_token, JwtTokenKind::ACCESS) {
        Ok(claim) => claim,
        Err(_) => {
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid access token"}"#);
        }
    };

    let user_id = claim.sub;

    match get_user_by_id(pool, &user_id).await {
        Ok(user) => {
            let mut map = HashMap::new();
            map.insert("username", user.username);
            map.insert("email", user.email);
            map.insert("phone_number", user.phone_number);
            map.insert("profile_pic", user.profile_pic.unwrap_or("".to_string()));
            map.insert("bio", user.bio.unwrap_or("".to_string()));

            let json_str = to_string(&map).unwrap();

            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(json_str)
        }
        Err(e) => {
            eprintln!(
                "{:?}: User profile fetching failed: {:?}",
                Utc::now().timestamp() as usize,
                e
            );

            HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"User not found"}"#)
        }
    }
}

#[patch("/profile")]
pub async fn patch_profile(
    pool: web::Data<PGPool>,
    req_body: web::Json<UpdateUser>,
    req: HttpRequest,
) -> impl Responder {
    println!(
        "{:?}: Update profile request from {:?}",
        Utc::now().timestamp() as usize,
        req.peer_addr()
    );

    // extract access token from cookie
    let access_token = match req.cookie("access_token") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"missing jwt token"}"#);
        }
    };

    // decode and validate JWT token
    let claim = match decode_jwt_token(access_token, JwtTokenKind::ACCESS) {
        Ok(claim) => claim,
        Err(_) => {
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid access token"}"#);
        }
    };

    let user_id = claim.sub;

    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(value) => value,
        Err(_) => {
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid access token"}"#);
        }
    };

    let mut data = req_body.into_inner();

    if let Some(username) = data.username.as_mut() {
        *username = username.trim().to_string();

        if !validate_username(username.clone()) {
            return HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid username format"}"#);
        }
    }

    if let Some(email) = data.email.as_mut() {
        *email = email.trim().to_string();

        if !validate_email(email.clone()) {
            return HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid email format"}"#);
        }
    }

    if let Some(phone_number) = data.phone_number.as_mut() {
        *phone_number = phone_number.trim().to_string();

        if !validate_phone_number(phone_number.clone()) {
            return HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid phone number format"}"#);
        }
    }

    if let Some(bio) = data.bio.as_mut() {
        *bio = bio.trim().to_string();

        if !validate_bio(bio.clone()) {
            return HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid bio format"}"#);
        }
    }

    if let Some(profile_pic) = data.profile_pic.as_mut() {
        *profile_pic = profile_pic.trim().to_string();

        if !validate_profile_pic(profile_pic.clone()) {
            return HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid profile pic format"}"#);
        }
    }

    let changes = UpdateUser {
        username: data.username,
        email: data.email,
        phone_number: data.phone_number,
        bio: data.bio,
        profile_pic: data.profile_pic,
    };

    match apply_profile_update(pool, user_uuid, changes).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            eprintln!(
                "{:?}: Failed to update user: {:?}",
                Utc::now().timestamp(),
                e
            );
            HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .body(r#"{"detail":"failed to update user"}"#)
        }
    }
}

