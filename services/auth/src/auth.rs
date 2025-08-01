use actix_web::web;
use bcrypt;
use chrono::Utc;
use diesel::dsl::insert_into;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind as DieselDbError, Error as DieselError};
use diesel_async::RunQueryDsl;

use shared::database::PGPool;
use shared::models::User;

pub async fn authenticate_user(
    pool: web::Data<PGPool>,
    uname: &str,
    pass: &str,
) -> Result<User, DieselError> {
    use shared::models::User;
    use shared::schema::users::dsl::*;

    let mut conn = pool.get().await.map_err(|e| {
        eprintln!(
            "{:?}: Failed to acquire DB connection: {:?}",
            Utc::now().timestamp() as usize,
            e
        );
        DieselError::DatabaseError(DieselDbError::UnableToSendCommand, Box::new(e.to_string()))
    })?;

    let user_result = users
        .filter(username.eq(uname))
        .first::<User>(&mut conn)
        .await;

    match user_result {
        Ok(user) => match bcrypt::verify(pass, &user.password_hash) {
            Ok(true) => Ok(user),
            Ok(false) | Err(_) => Err(DieselError::NotFound),
        },
        Err(_) => Err(DieselError::NotFound),
    }
}

pub async fn add_user_to_db(
    pool: web::Data<PGPool>,
    username: &str,
    email: &str,
    phone_number: &str,
    password_hash: &str,
) -> Result<usize, DieselError> {
    use shared::models::RegisterUser;
    use shared::schema::users;

    let mut conn = pool.get().await.map_err(|e| {
        eprintln!(
            "{:?}: Failed to acquire DB connection: {:?}",
            Utc::now().timestamp() as usize,
            e
        );
        DieselError::DatabaseError(DieselDbError::UnableToSendCommand, Box::new(e.to_string()))
    })?;

    let new_user = RegisterUser {
        username: username.to_string(),
        email: email.to_string(),
        phone_number: phone_number.to_string(),
        password_hash: password_hash.to_string(),
    };

    insert_into(users::table)
        .values(&new_user)
        .execute(&mut conn)
        .await
}
