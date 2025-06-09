use actix_web::web;
use chrono::Utc;
use diesel::{ExpressionMethods, query_dsl::methods::FilterDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::database::PGPool;
use super::models::{UpdateUser, User};
use diesel::result::DatabaseErrorKind as DieselDbError;
use diesel::result::Error as DieselError;

pub async fn get_user_by_id(pool: web::Data<PGPool>, user_id: &str) -> Result<User, DieselError> {
    use crate::models::User;
    use crate::schema::users::dsl::*;

    let mut conn = pool.get().await.map_err(|e| {
        eprintln!(
            "{:?}: Failed to acquire DB connection: {:?}",
            Utc::now().timestamp() as usize,
            e
        );
        DieselError::DatabaseError(DieselDbError::UnableToSendCommand, Box::new(e.to_string()))
    })?;

    let parsed_uuid = Uuid::parse_str(user_id).map_err(|e| {
        eprintln!(
            "{:?}: Failed to acquire DB connection: {:?}",
            Utc::now().timestamp() as usize,
            e
        );
        DieselError::DatabaseError(DieselDbError::UnableToSendCommand, Box::new(e.to_string()))
    })?;

    let user_result = users
        .filter(id.eq(parsed_uuid))
        .first::<User>(&mut conn)
        .await;

    match user_result {
        Ok(user) => Ok(user),
        Err(_) => Err(DieselError::NotFound),
    }
}

pub async fn apply_profile_update(
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
