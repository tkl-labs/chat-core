use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse, Responder, get, patch, web};
use base64::prelude::*;
use chrono::Utc;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use image::load_from_memory;
use regex::Regex;
use uuid::Uuid;

use crate::actix::auth::jwt::{JwtTokenKind, decode_jwt_token};
use crate::database::init::PGPool;
use crate::models::UpdateUser;
use diesel::result::DatabaseErrorKind as DieselDbError;
use diesel::result::Error as DieselError;

use std::collections::HashMap;
use serde_json::to_string;

const EMAIL_REGEX: &str = r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$";
const PHONE_NUMBER_REGEX: &str = r"^\+?[0-9]{7,15}$";

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
    let claim = match decode_jwt_token(access_token) {
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
            
            print!(
                "{:?}: User profile fetched successfully: {:?}",    
                Utc::now().timestamp() as usize,
                user_id
            );
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

        // sanitise username
        let username_meets_requirements = (username.len() >= 8 && username.len() <= 16)
            && (username.chars().all(char::is_alphanumeric));

        if !username_meets_requirements {
            return HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid username format"}"#);
        }
    }

    if let Some(email) = data.email.as_mut() {
        *email = email.trim().to_string();

        // sanitise email
        let email_re = Regex::new(EMAIL_REGEX).unwrap();
        let email_meets_requirements = email_re.is_match(&email);

        if !email_meets_requirements {
            return HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid email format"}"#);
        }
    }

    if let Some(phone_number) = data.phone_number.as_mut() {
        *phone_number = phone_number.trim().to_string();

        // sanitise phone number
        let phone_re = Regex::new(PHONE_NUMBER_REGEX).unwrap();
        let phone_number_meets_requirements = phone_re.is_match(&phone_number);

        if !phone_number_meets_requirements {
            return HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid phone number format"}"#);
        }
    }

    if let Some(bio) = data.bio.as_mut() {
        *bio = bio.trim().to_string();

        // sanitise bio
        let bio_meets_requirements = bio.len() >= 1 && bio.len() <= 500;

        if !bio_meets_requirements {
            return HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid bio format"}"#);
        }
    }

    if let Some(profile_pic) = data.profile_pic.as_mut() {
        *profile_pic = profile_pic.trim().to_string();

        // sanitise profile pic
        let profile_pic_meets_requirements = match BASE64_STANDARD.decode(profile_pic) {
            Ok(bytes) => match load_from_memory(&bytes) {
                Ok(_) => true,   // successfully decoded and parsed as an image
                Err(_) => false, // not a valid image
            },
            Err(_) => false, // not valid base64
        };

        if !profile_pic_meets_requirements {
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

    match apply_user_update(pool, user_uuid, changes).await {
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

async fn apply_user_update(
    pool: web::Data<PGPool>,
    user_uuid: Uuid,
    changes: UpdateUser,
) -> Result<bool, DieselError> {
    use crate::schema::users::dsl::users;
    use crate::schema::users::*;

    let mut conn = pool.get().await.map_err(|e| {
        eprintln!(
            "{:?}: Failed to acquire DB connection: {:?}",
            Utc::now().timestamp(),
            e
        );
        DieselError::DatabaseError(DieselDbError::UnableToSendCommand, Box::new(e.to_string()))
    })?;

    diesel::update(users)
        .set(&changes)
        .filter(id.eq(user_uuid))
        .execute(&mut conn)
        .await?;

    Ok(true)
}
