use actix_web::http::header::ContentType;
use actix_web::web;
use actix_web::{HttpRequest, HttpResponse, Responder, get, patch, post};
use chrono::Utc;
use serde::Deserialize;

use shared::csrf::verify_csrf_token;
use shared::database::PGPool;
use shared::jwt::{JwtTokenKind, extract_user_id};
use shared::validate::validate_existing_username;

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

use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind as DieselDbError, Error as DieselError};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use shared::models::{CreateFriend, User};
use shared::schema::friend_request::dsl::friend_request;
use shared::schema::users;

#[derive(Debug)]
pub enum AddFriendResult {
    Created,
    AlreadyExists,
    AlreadyFriends,
}

pub async fn send_friend_request(
    pool: web::Data<PGPool>,
    requesting_user_id: &str,
    receiver_username: &str,
) -> HttpResponse {
    match add_friend_request_to_db(pool, requesting_user_id, receiver_username).await {
        Ok(AddFriendResult::Created) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(r#"{"detail":"friend request sent successfully"}"#),
        Ok(AddFriendResult::AlreadyExists) => HttpResponse::BadRequest()
            .content_type(ContentType::json())
            .body(r#"{"detail":"friend request already exists"}"#),
        Ok(AddFriendResult::AlreadyFriends) => HttpResponse::BadRequest()
            .content_type(ContentType::json())
            .body(r#"{"detail":"already friends with this user"}"#),
        Err(_) => HttpResponse::InternalServerError()
            .content_type(ContentType::json())
            .body(r#"{"detail":"could not send friend request"}"#),
    }
}

pub async fn add_friend_request_to_db(
    pool: web::Data<PGPool>,
    requesting_id: &str,
    responding_username: &str,
) -> Result<AddFriendResult, DieselError> {
    use diesel::insert_into;
    use shared::models::{CreateFriendRequest, User};
    use shared::schema::friend::dsl as f;
    use shared::schema::users::dsl as u;

    let mut conn = pool.get().await.map_err(|e| {
        eprintln!("{}: Failed to get DB connection: {:?}", Utc::now(), e);
        DieselError::NotFound // Ideally map to a proper custom error
    })?;

    let user_uuid = Uuid::parse_str(requesting_id).map_err(|e| {
        eprintln!("{}: Invalid user UUID: {:?}", Utc::now(), e);
        DieselError::NotFound
    })?;

    let receiver_user: User = u::users
        .filter(u::username.ilike(responding_username))
        .first::<User>(&mut conn)
        .await?;

    if receiver_user.id == user_uuid {
        return Ok(AddFriendResult::AlreadyExists); // Cannot friend yourself
    }

    let is_already_friends = match f::friend
        .filter(
            f::user1
                .eq(user_uuid)
                .and(f::user2.eq(receiver_user.id))
                .or(f::user1.eq(receiver_user.id).and(f::user2.eq(user_uuid))),
        )
        .select(f::user1)
        .first::<Uuid>(&mut conn)
        .await
    {
        Ok(_) => true,
        Err(DieselError::NotFound) => false,
        Err(e) => return Err(e.into()),
    };

    if is_already_friends {
        return Ok(AddFriendResult::AlreadyFriends);
    }

    let new_friend_request = vec![CreateFriendRequest {
        requester: user_uuid,
        receiver: receiver_user.id,
    }];

    let result = insert_into(friend_request)
        .values(&new_friend_request)
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .await?;

    if result > 0 {
        Ok(AddFriendResult::Created)
    } else {
        Ok(AddFriendResult::AlreadyExists)
    }
}

pub async fn get_all_friends(
    pool: web::Data<PGPool>,
    fetching_user_id: &str,
) -> Result<String, DieselError> {
    use shared::models::User;
    use shared::schema::{friend, users};

    let mut conn = pool.get().await.map_err(|e| {
        eprintln!(
            "{:?}: Failed to acquire DB connection: {:?}",
            Utc::now().timestamp() as usize,
            e
        );
        DieselError::DatabaseError(DieselDbError::UnableToSendCommand, Box::new(e.to_string()))
    })?;

    let user_uuid = Uuid::parse_str(&fetching_user_id.to_string()).map_err(|e| {
        eprintln!(
            "{:?}: Failed to acquire DB connection: {:?}",
            Utc::now().timestamp() as usize,
            e
        );
        DieselError::DatabaseError(DieselDbError::UnableToSendCommand, Box::new(e.to_string()))
    })?;

    let results: Vec<User> = users::table
        .inner_join(
            friend::table.on(users::id
                .eq(friend::user1)
                .and(friend::user2.eq(user_uuid))
                .or(users::id.eq(friend::user2).and(friend::user1.eq(user_uuid)))),
        )
        .select(users::all_columns)
        .distinct()
        .load(&mut conn)
        .await?;

    serde_json::to_string_pretty(&results).map_err(|e| {
        eprintln!("JSON serialization error: {:?}", e);
        DieselError::SerializationError(Box::new(e))
    })
}

pub async fn get_all_friend_requests(
    pool: web::Data<PGPool>,
    user_id: &str,
) -> Result<String, DieselError> {
    use shared::schema::friend_request::dsl as fr;
    use shared::schema::users::dsl as u;

    let mut conn = pool.get().await.map_err(|e| {
        eprintln!(
            "{:?}: Failed to acquire DB connection: {:?}",
            Utc::now().timestamp() as usize,
            e
        );
        DieselError::DatabaseError(DieselDbError::UnableToSendCommand, Box::new(e.to_string()))
    })?;

    let user_uuid = Uuid::parse_str(user_id).map_err(|_| DieselError::NotFound)?;

    let results: Vec<User> = u::users
        .inner_join(friend_request.on(fr::requester.eq(users::id)))
        .filter(fr::receiver.eq(user_uuid))
        .select(users::all_columns)
        .load(&mut conn)
        .await?;

    serde_json::to_string_pretty(&results).map_err(|e| {
        eprintln!("JSON serialization error: {:?}", e);
        DieselError::SerializationError(Box::new(e))
    })
}

pub async fn remove_friend(
    pool: web::Data<PGPool>,
    user_id: &str,
    removed_friend_id: &str,
) -> HttpResponse {
    use shared::schema::friend::dsl as f;

    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "{}: Failed to acquire DB connection: {:?}",
                Utc::now().timestamp(),
                e
            );
            return HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(r#"{"detail":"Database connection error"}"#);
        }
    };

    let user_uuid = match Uuid::parse_str(user_id) {
        Ok(uuid) => uuid,
        Err(e) => {
            eprintln!("{}: Invalid user_id UUID: {:?}", Utc::now().timestamp(), e);
            return HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .body(r#"{"detail":"Invalid user_id UUID"}"#);
        }
    };

    let removed_friend_uuid = match Uuid::parse_str(removed_friend_id) {
        Ok(uuid) => uuid,
        Err(e) => {
            eprintln!(
                "{}: Invalid removed_friend_id UUID: {:?}",
                Utc::now().timestamp(),
                e
            );
            return HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .body(r#"{"detail":"Invalid removed_friend_id UUID"}"#);
        }
    };

    let rows_deleted = diesel::delete(
        f::friend.filter(
            f::user1
                .eq(user_uuid)
                .and(f::user2.eq(removed_friend_uuid))
                .or(f::user1.eq(removed_friend_uuid).and(f::user2.eq(user_uuid))),
        ),
    )
    .execute(&mut conn)
    .await;

    match rows_deleted {
        Ok(_) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(r#"{"detail":"friend removed successfully"}"#),
        Err(e) => {
            eprintln!("DB insert error: {:?}", e);
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(r#"{"detail":"internal server error"}"#)
        }
    }
}

pub async fn update_friend_request(
    pool: web::Data<PGPool>,
    responding_user_id: &str,
    requesting_user_id: &str,
    accept: bool,
) -> HttpResponse {
    use shared::schema::friend::dsl as f;
    use shared::schema::friend_request::dsl as fr;

    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "{}: Failed to acquire DB connection: {:?}",
                Utc::now().timestamp(),
                e
            );
            return HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(r#"{"detail":"Database connection error"}"#);
        }
    };

    let responding_uuid = match Uuid::parse_str(responding_user_id) {
        Ok(uuid) => uuid,
        Err(e) => {
            eprintln!(
                "{}: Invalid responding_user_id UUID: {:?}",
                Utc::now().timestamp(),
                e
            );
            return HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .body(r#"{"detail":"Invalid responding_user_id UUID"}"#);
        }
    };

    let requesting_uuid = match Uuid::parse_str(requesting_user_id) {
        Ok(uuid) => uuid,
        Err(e) => {
            eprintln!(
                "{}: Invalid requesting_user_id UUID: {:?}",
                Utc::now().timestamp(),
                e
            );
            return HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .body(r#"{"detail":"Invalid requesting_user_id UUID"}"#);
        }
    };

    let rows_deleted = diesel::delete(
        friend_request.filter(
            fr::receiver
                .eq(responding_uuid)
                .and(fr::requester.eq(requesting_uuid)),
        ),
    )
    .execute(&mut conn)
    .await;

    match rows_deleted {
        Ok(_) => {}
        Err(e) => {
            eprintln!("DB insert error: {:?}", e);
            return HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(r#"{"detail":"internal server error"}"#);
        }
    }

    if accept {
        let new_friend = CreateFriend {
            user1: requesting_uuid,
            user2: responding_uuid,
        };

        let rows_inserted = diesel::insert_into(f::friend)
            .values(&new_friend)
            .execute(&mut conn)
            .await;

        match rows_inserted {
            Ok(n) if n > 0 => HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(r#"{"detail":"friend request accepted"}"#),
            Ok(_) => HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(r#"{"detail":"failed to accept friend request"}"#),
            Err(e) => {
                eprintln!("DB insert error: {:?}", e);
                HttpResponse::InternalServerError()
                    .content_type(ContentType::json())
                    .body(r#"{"detail":"internal server error"}"#)
            }
        }
    } else {
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(r#"{"detail":"friend request declined"}"#)
    }
}
