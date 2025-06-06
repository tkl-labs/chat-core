use actix_web::web;
use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse, Responder, delete, get, post};
use chrono::Utc;
use serde::Deserialize;

use crate::services::friendship::add_friend;
use crate::services::jwt::{
    JwtError, extract_user_id_from_jwt_token,
};
use crate::services::validate::validate_existing_username;

#[derive(Deserialize)]
struct AddFriendForm {
    username: String,
}

#[delete("/remove")]
pub async fn delete_remove(req: HttpRequest, req_body: web::Json<AddFriendForm>) -> impl Responder {
    println!(
        "{:?}: Delete friend request from {:?}",
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

    let user_id = match extract_user_id_from_jwt_token(access_token) {
        Ok(value) => value,
        Err(JwtError::Expired) => {
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"access token expired"}"#);
        }
        Err(JwtError::Invalid) => {
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid access token"}"#);
        }
        Err(JwtError::Other(e)) => {
            eprintln!("JWT error: {:?}", e);
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"token verification failed"}"#);
        }
    };

    let username = req_body.username.trim();

    if !validate_existing_username(&username) {
        return HttpResponse::Unauthorized()
            .content_type(ContentType::json())
            .body(r#"{"detail":"invalid username"}"#);
    }

    println!("user_id: {:?}", user_id);

    let successful = add_friend(&user_id, &username);

    if successful {

    }
    HttpResponse::Ok().body("")
}

#[get("/all")]
pub async fn get_all(req: HttpRequest) -> impl Responder {
    println!(
        "{:?}: Delete friend request from {:?}",
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

    let user_uuid = match extract_user_id_from_jwt_token(access_token) {
        Ok(value) => value,
        Err(JwtError::Expired) => {
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"access token expired"}"#);
        }
        Err(JwtError::Invalid) => {
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid access token"}"#);
        }
        Err(JwtError::Other(e)) => {
            eprintln!("JWT error: {:?}", e);
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"token verification failed"}"#);
        }
    };

    println!("user_uuid: {:?}", user_uuid);

    HttpResponse::Ok().body("")
}

#[post("/add")]
pub async fn post_add(req: HttpRequest) -> impl Responder {
    println!(
        "{:?}: Delete friend request from {:?}",
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

    let user_uuid = match extract_user_id_from_jwt_token(access_token) {
        Ok(value) => value,
        Err(JwtError::Expired) => {
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"access token expired"}"#);
        }
        Err(JwtError::Invalid) => {
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid access token"}"#);
        }
        Err(JwtError::Other(e)) => {
            eprintln!("JWT error: {:?}", e);
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"token verification failed"}"#);
        }
    };

    println!("user_uuid: {:?}", user_uuid);

    HttpResponse::Ok().body("")
}
