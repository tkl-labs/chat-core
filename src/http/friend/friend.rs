use actix_web::http::header::ContentType;
use actix_web::web;
use actix_web::{HttpRequest, HttpResponse, Responder, get, patch, post};
use chrono::Utc;
use serde::Deserialize;

use crate::db::operations::PGPool;
use crate::services::csrf::verify_csrf_token;
use crate::services::friend::{
    get_all_friend_requests, get_all_friends, remove_friend, send_friend_request,
    update_friend_request,
};
use crate::services::jwt::{JwtTokenKind, extract_user_id};
use crate::services::validate::validate_existing_username;

#[derive(Deserialize)]
struct AddFriendForm {
    username: String,
}

#[derive(Deserialize)]
struct RemoveFriendForm {
    removed_friend_id: String,
}

#[derive(Deserialize)]
struct FriendRequestForm {
    requesting_user_id: String,
    accept: bool,
}

#[post("/remove")]
pub async fn post_remove(
    pool: web::Data<PGPool>,
    req: HttpRequest,
    req_body: web::Json<RemoveFriendForm>,
) -> impl Responder {
    println!(
        "{:?}: POST /friend/remove from {:?}",
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

    let removed_friend_id = req_body.removed_friend_id.trim();

    remove_friend(pool, &user_id, removed_friend_id).await
}

#[get("/all")]
pub async fn get_all(pool: web::Data<PGPool>, req: HttpRequest) -> impl Responder {
    println!(
        "{:?}: GET /friend/all from {:?}",
        Utc::now().timestamp() as usize,
        req.peer_addr()
    );

    // extract user id from access token
    let user_id = match extract_user_id(&req, JwtTokenKind::ACCESS) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let all_friends_json = match get_all_friends(pool, &user_id).await {
        Ok(val) => val,
        Err(_) => "{}".to_string(),
    };

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(all_friends_json)
}

#[post("/add")]
pub async fn post_add(
    pool: web::Data<PGPool>,
    req: HttpRequest,
    req_body: web::Json<AddFriendForm>,
) -> impl Responder {
    println!(
        "{:?}: POST /friend/add from {:?}",
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

    let username = req_body.username.trim();

    if !validate_existing_username(&username) {
        return HttpResponse::Unauthorized()
            .content_type(ContentType::json())
            .body(r#"{"detail":"invalid username"}"#);
    }

    send_friend_request(pool, &user_id, &username).await
}

#[patch("/add")]
pub async fn patch_add(
    pool: web::Data<PGPool>,
    req: HttpRequest,
    req_body: web::Json<FriendRequestForm>,
) -> impl Responder {
    println!(
        "{:?}: PATCH /friend/add from {:?}",
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
    let responding_user_id = match extract_user_id(&req, JwtTokenKind::ACCESS) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let requesting_user_id = req_body.requesting_user_id.trim();
    let accept = req_body.accept;

    update_friend_request(pool, &responding_user_id, &requesting_user_id, accept).await
}

#[get("/requests")]
pub async fn get_friend_requests(pool: web::Data<PGPool>, req: HttpRequest) -> impl Responder {
    let user_id = match extract_user_id(&req, JwtTokenKind::ACCESS) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    match get_all_friend_requests(pool, &user_id).await {
        Ok(requests) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(requests),
        Err(_) => HttpResponse::InternalServerError()
            .content_type(ContentType::json())
            .body("failed to get friend requests"),
    }
}
