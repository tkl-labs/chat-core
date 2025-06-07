use actix_web::web;
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind as DieselDbError, Error as DieselError};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::operations::PGPool;
use crate::models::{CreateFriend, User};
use crate::schema::{friend, users};
use crate::schema::friend_request::dsl::friend_request;

#[derive(Debug)]
pub enum AddFriendResult {
    Created,
    AlreadyExists,
}

pub async fn send_friend_request(pool: web::Data<PGPool>, requesting_user_id: &str, receiver_username: &str) -> bool {
    match add_friend_request_to_db(pool, requesting_user_id, receiver_username).await {
        Ok(AddFriendResult::Created) => true, // friend request successfully created
        Ok(AddFriendResult::AlreadyExists) => false, // friend request already exists
        Err(_) => false, // all other errors
    }
}

pub async fn add_friend_request_to_db(
    pool: web::Data<PGPool>,
    requesting_id: &str,
    responding_username: &str,
) -> Result<AddFriendResult, DieselError> {
    use crate::models::{CreateFriendRequest, User};
    use crate::schema::friend_request::dsl::*;
    use crate::schema::users::dsl::*;
    use diesel::insert_into;

    let mut conn = pool.get().await.map_err(|e| {
        eprintln!("{}: Failed to get DB connection: {:?}", Utc::now(), e);
        DieselError::NotFound // Ideally map to a proper custom error
    })?;

    let user_uuid = Uuid::parse_str(requesting_id).map_err(|e| {
        eprintln!("{}: Invalid user UUID: {:?}", Utc::now(), e);
        DieselError::NotFound
    })?;

    let receiver_user: User = users
        .filter(username.ilike(responding_username))
        .first::<User>(&mut conn)
        .await?;

    if receiver_user.id == user_uuid {
        return Ok(AddFriendResult::AlreadyExists); // Cannot friend yourself
    }

    let new_friend_request = vec![
        CreateFriendRequest {
            requester: user_uuid,
            receiver: receiver_user.id,
        },
    ];

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
        .inner_join(friend::table.on(
            friend::user1.eq(users::id).or(friend::user2.eq(users::id)),
        ))
        .filter(friend::user1.eq(user_uuid).or(friend::user2.eq(user_uuid)))
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

pub async fn update_friend_request(
    pool: web::Data<PGPool>,
    responding_user_id: &str,
    requesting_user_id: &str,
    accept: bool,
) -> Result<bool, DieselError> {
    use crate::schema::friend_request::dsl as fr;
    use crate::schema::friend::dsl as f;

    let mut conn = pool.get().await.map_err(|e| {
        eprintln!(
            "{:?}: Failed to acquire DB connection: {:?}",
            Utc::now().timestamp() as usize,
            e
        );
        DieselError::DatabaseError(DieselDbError::UnableToSendCommand, Box::new(e.to_string()))
    })?;

    let responding_uuid = Uuid::parse_str(responding_user_id).map_err(|_| DieselError::NotFound)?;
    let requesting_uuid = Uuid::parse_str(requesting_user_id).map_err(|_| DieselError::NotFound)?;

    diesel::delete(
                fr::friend_request.filter(
                    fr::receiver
                        .eq(responding_uuid)
                        .and(fr::requester.eq(requesting_uuid)),
                ),
            )
            .execute(&mut conn)
            .await?;

    if accept {
        let new_friend = CreateFriend {
            user1: requesting_uuid,
            user2: responding_uuid,
        };

        diesel::insert_into(f::friend)
            .values(&new_friend)
            .execute(&mut conn)
            .await?;
    }
    Ok(true)
}
