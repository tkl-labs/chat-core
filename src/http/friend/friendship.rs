use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse, Responder, delete, get, post};
use chrono::Utc;

use crate::services::jwt::{
    JwtError, extract_user_id_from_jwt_token,
};

#[delete("/remove")]
pub async fn delete_remove(req: HttpRequest) -> impl Responder {
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
