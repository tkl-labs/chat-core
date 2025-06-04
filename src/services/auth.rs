use actix_web::web;
use bcrypt;
use chrono::Utc;
use diesel::dsl::insert_into;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind as DieselDbError, Error as DieselError};
use diesel_async::RunQueryDsl;

use crate::db::operations::PGPool;
use crate::models::User;

pub async fn authenticate_user(
    pool: web::Data<PGPool>,
    uname: &str,
    pass: &str,
) -> Result<User, DieselError> {
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
    use crate::models::RegisterUser;
    use crate::schema::users;

    let mut conn = pool.get().await.map_err(|e| {
        eprintln!(
            "{:?}: Failed to acquire DB connection: {:?}",
            Utc::now().timestamp() as usize,
            e
        );
        DieselError::DatabaseError(DieselDbError::UnableToSendCommand, Box::new(e.to_string()))
    })?;

    let u = username.to_string();
    let e = email.to_string();
    let pn = phone_number.to_string();
    let ph = password_hash.to_string();

    let new_user = RegisterUser {
        username: u,
        email: e,
        phone_number: pn,
        password_hash: ph,
    };

    insert_into(users::table)
        .values(&new_user)
        .execute(&mut conn)
        .await
}
