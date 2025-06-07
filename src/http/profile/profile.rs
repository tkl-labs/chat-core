use std::collections::HashMap;

use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse, Responder, get, patch, web};
use chrono::Utc;
use serde_json::to_string;
use uuid::Uuid;

use crate::db::operations::PGPool;
use crate::models::UpdateUser;
use crate::services::csrf::verify_csrf_token;
use crate::services::jwt::{extract_user_id, JwtTokenKind};
use crate::services::profile::{apply_profile_update, get_user_by_id};
use crate::services::validate::{
    validate_bio, validate_email, validate_new_username, validate_phone_number,
    validate_profile_pic,
};

#[get("/self")]
pub async fn get_profile(pool: web::Data<PGPool>, req: HttpRequest) -> impl Responder {
    println!(
        "{:?}: GET /profile/self from {:?}",
        Utc::now().timestamp() as usize,
        req.peer_addr()
    );

    // extract user id from access token
    let user_id = match extract_user_id(&req, JwtTokenKind::ACCESS) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

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

#[patch("/self")]
pub async fn patch_profile(
    pool: web::Data<PGPool>,
    req_body: web::Json<UpdateUser>,
    req: HttpRequest,
) -> impl Responder {
    println!(
        "{:?}: PATCH /profile/self from {:?}",
        Utc::now().timestamp() as usize,
        req.peer_addr()
    );

    let verify_csrf = verify_csrf_token(&req);

    if !verify_csrf {
        return HttpResponse::Unauthorized()
            .content_type(ContentType::json())
            .body(r#"{"detail":"csrf failed"}"#);
    }

    // extract user id from access token
    let user_id = match extract_user_id(&req, JwtTokenKind::ACCESS) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

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

        match validate_new_username(pool.clone(), &username).await {
            Ok(valid) => {
                if !valid {
                    return HttpResponse::BadRequest()
                        .content_type(ContentType::json())
                        .body(r#"{"detail":"invalid username format"}"#);
                }
            }
            Err(_) => {
                return HttpResponse::BadRequest()
                    .content_type(ContentType::json())
                    .body(r#"{"detail":"username taken"}"#);
            }
        };
    }

    if let Some(email) = data.email.as_mut() {
        *email = email.trim().to_string();

        match validate_email(pool.clone(), &email).await {
            Ok(valid) => {
                if !valid {
                    return HttpResponse::BadRequest()
                        .content_type(ContentType::json())
                        .body(r#"{"detail":"invalid email format"}"#);
                }
            }
            Err(_) => {
                return HttpResponse::BadRequest()
                    .content_type(ContentType::json())
                    .body(r#"{"detail":"email taken"}"#);
            }
        };
    }

    if let Some(phone_number) = data.phone_number.as_mut() {
        *phone_number = phone_number.trim().to_string();

        match validate_phone_number(pool.clone(), &phone_number).await {
            Ok(valid) => {
                if !valid {
                    return HttpResponse::BadRequest()
                        .content_type(ContentType::json())
                        .body(r#"{"detail":"invalid phone number format"}"#);
                }
            }
            Err(_) => {
                return HttpResponse::BadRequest()
                    .content_type(ContentType::json())
                    .body(r#"{"detail":"phone number taken"}"#);
            }
        };
    }

    if let Some(bio) = data.bio.as_mut() {
        *bio = bio.trim().to_string();

        if !validate_bio(&bio) {
            return HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid bio format"}"#);
        }
    }

    if let Some(profile_pic) = data.profile_pic.as_mut() {
        *profile_pic = profile_pic.trim().to_string();

        if !validate_profile_pic(&profile_pic) {
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
