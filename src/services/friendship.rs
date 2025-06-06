use actix_web::web;
use chrono::Utc;
use diesel::dsl::insert_into;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind as DieselDbError, Error as DieselError};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::operations::PGPool;

pub async fn add_friend(pool: web::Data<PGPool>, user_id: &str, friend_username: &str) -> bool {
    match add_friendship_to_db(pool, user_id, friend_username).await {
        Ok(_) => true,
        Err(_) => false
    }
}

pub async fn add_friendship_to_db(
    pool: web::Data<PGPool>,
    user_id: &str,
    friend_username: &str,
) -> Result<usize, DieselError> {
    use crate::models::{CreateFriendship, User};
    use crate::schema::friendships;
    use crate::schema::users::dsl::*;

    let mut conn = pool.get().await.map_err(|e| {
        eprintln!(
            "{:?}: Failed to acquire DB connection: {:?}",
            Utc::now().timestamp() as usize,
            e
        );
        DieselError::DatabaseError(DieselDbError::UnableToSendCommand, Box::new(e.to_string()))
    })?;

    let user_uuid = Uuid::parse_str(user_id).map_err(|e| {
        eprintln!(
            "{:?}: Failed to acquire DB connection: {:?}",
            Utc::now().timestamp() as usize,
            e
        );
        DieselError::DatabaseError(DieselDbError::UnableToSendCommand, Box::new(e.to_string()))
    })?;

    let user_result = users
        .filter(username.ilike(friend_username))
        .first::<User>(&mut conn)
        .await;

    let friend = match user_result {
        Ok(user) => user,
        Err(_) => return Err(DieselError::NotFound),
    };

    let friend_uuid = Uuid::parse_str(&friend.id.to_string()).map_err(|e| {
        eprintln!(
            "{:?}: Failed to acquire DB connection: {:?}",
            Utc::now().timestamp() as usize,
            e
        );
        DieselError::DatabaseError(DieselDbError::UnableToSendCommand, Box::new(e.to_string()))
    })?;

    let new_friendship_1 = CreateFriendship {
        user_id: user_uuid,
        friend_id: friend_uuid,
        friendship_status: "pending".to_string(),
    };

    let new_friendship_2 = CreateFriendship {
        user_id: friend_uuid,
        friend_id: user_uuid,
        friendship_status: "pending".to_string(),
    };

    insert_into(friendships::table)
        .values(vec![&new_friendship_1, &new_friendship_2])
        .execute(&mut conn)
        .await
}

pub async fn get_all_friends(pool: web::Data<PGPool>, fetching_user_id: &str) -> Result<String, DieselError> {
    use crate::models::User;
    use crate::schema::{users, friendships};

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
        .select(users::all_columns)
        .load(&mut conn)
        .await?;

    serde_json::to_string_pretty(&results)
        .map_err(|e| {
            eprintln!("JSON serialization error: {:?}", e);
            DieselError::SerializationError(Box::new(e))
        })
}
