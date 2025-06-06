use actix_web::web;
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind as DieselDbError, Error as DieselError};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::operations::PGPool;

#[derive(Debug)]
pub enum AddFriendResult {
    Created,
    AlreadyExists,

}

pub async fn add_friend(pool: web::Data<PGPool>, user_id: &str, friend_username: &str) -> bool {
    match add_friendship_to_db(pool, user_id, friend_username).await {
        Ok(AddFriendResult::Created) => true, // friend request successfully created
        Ok(AddFriendResult::AlreadyExists) => false, // friend request already exists
        Err(_) => false, // all other errors
    }
}

pub async fn add_friendship_to_db(
    pool: web::Data<PGPool>,
    requesting_id: &str,
    responding_username: &str,
) -> Result<AddFriendResult, DieselError> {
    use crate::models::{CreateFriendship, User};
    use crate::schema::friendships::dsl::*;
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

    let friend: User = users
        .filter(username.ilike(responding_username))
        .first::<User>(&mut conn)
        .await?;

    if friend.id == user_uuid {
        return Ok(AddFriendResult::AlreadyExists); // Cannot friend yourself
    }

    let new_friendships = vec![
        CreateFriendship {
            user_id: user_uuid,
            friend_id: friend.id,
            friendship_status: "pending".to_string(),
        },
        CreateFriendship {
            user_id: friend.id,
            friend_id: user_uuid,
            friendship_status: "pending".to_string(),
        },
    ];

    let result = insert_into(friendships)
        .values(&new_friendships)
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
    use crate::schema::{friendships, users};

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
        .inner_join(friendships::table.on(friendships::user_id.eq(users::id)))
        .filter(users::id.ne(user_uuid))
        .filter(friendships::friendship_status.eq("accepted"))
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
    use crate::schema::friendships::dsl::*;

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

    if accept {
        diesel::update(
            friendships.filter(
                (user_id
                    .eq(responding_uuid)
                    .and(friend_id.eq(requesting_uuid)))
                .or(user_id
                    .eq(requesting_uuid)
                    .and(friend_id.eq(responding_uuid))),
            ),
        )
        .set(friendship_status.eq("accepted"))
        .execute(&mut conn)
        .await?;
    } else {
        diesel::delete(
            friendships.filter(
                (user_id
                    .eq(responding_uuid)
                    .and(friend_id.eq(requesting_uuid)))
                .or(user_id
                    .eq(requesting_uuid)
                    .and(friend_id.eq(responding_uuid))),
            ),
        )
        .execute(&mut conn)
        .await?;
    }

    Ok(true)
}
