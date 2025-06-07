use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, web};
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind as DieselDbError, Error as DieselError};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::operations::PGPool;
use crate::models::{CreateFriend, User};
use crate::schema::friend_request::dsl::friend_request;
use crate::schema::users;

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
    use crate::models::{CreateFriendRequest, User};
    use crate::schema::friend::dsl as f;
    use crate::schema::users::dsl as u;
    use diesel::insert_into;

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
    use crate::models::User;
    use crate::schema::{friend, users};

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
        .inner_join(friend::table.on(friend::user1.eq(user_uuid).or(friend::user2.eq(user_uuid))))
        .filter(friend::user1.eq(user_uuid).or(friend::user2.eq(user_uuid)))
        .filter(users::id.ne(user_uuid))
        .select(users::all_columns)
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
    use crate::schema::friend_request::dsl as fr;
    use crate::schema::users::dsl as u;

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
    use crate::schema::friend::dsl as f;

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
    use crate::schema::friend::dsl as f;
    use crate::schema::friend_request::dsl as fr;

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
