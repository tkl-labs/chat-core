use actix_web::web;
use base64::prelude::*;
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind as DieselDbError, Error as DieselError};
use diesel_async::RunQueryDsl;
use image::load_from_memory;
use regex::Regex;

use super::database::PGPool;

const LOWERCASE_REGEX: &str = "[a-z]";
const UPPERCASE_REGEX: &str = "[A-Z]";
const NUMERIC_REGEX: &str = "[0-9]";
const SPECIAL_REGEX: &str = "[^a-zA-Z0-9]";
const EMAIL_REGEX: &str = r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$";
const PHONE_NUMBER_REGEX: &str = r"^\+?[0-9]{7,15}$";

pub fn validate_existing_username(username: &str) -> bool {
    let valid_username = (username.len() >= 8 && username.len() <= 16)
        && (username.chars().all(char::is_alphanumeric));

    valid_username
}

pub async fn validate_new_username(
    pool: web::Data<PGPool>,
    new_username: &str,
) -> Result<bool, DieselError> {
    let valid_username = (new_username.len() >= 8 && new_username.len() <= 16)
        && (new_username.chars().all(char::is_alphanumeric));

    if valid_username {
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
            .filter(username.ilike(new_username))
            .first::<User>(&mut conn)
            .await;

        if user_result.is_ok() {
            Err(DieselError::DatabaseError(
                DieselDbError::UniqueViolation,
                Box::new("username taken".to_string()),
            ))
        } else {
            Ok(true)
        }
    } else {
        Ok(false)
    }
}

pub async fn validate_email(pool: web::Data<PGPool>, new_email: &str) -> Result<bool, DieselError> {
    let email_re = Regex::new(EMAIL_REGEX).unwrap();
    let valid_email = email_re.is_match(&new_email);

    if valid_email {
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
            .filter(email.ilike(new_email))
            .first::<User>(&mut conn)
            .await;

        if user_result.is_ok() {
            Err(DieselError::DatabaseError(
                DieselDbError::UniqueViolation,
                Box::new("email taken".to_string()),
            ))
        } else {
            Ok(true)
        }
    } else {
        Ok(false)
    }
}

pub async fn validate_phone_number(
    pool: web::Data<PGPool>,
    new_phone_number: &str,
) -> Result<bool, DieselError> {
    let phone_re = Regex::new(PHONE_NUMBER_REGEX).unwrap();
    let valid_phone_number = phone_re.is_match(new_phone_number);

    if valid_phone_number {
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
            .filter(phone_number.eq(new_phone_number))
            .first::<User>(&mut conn)
            .await;

        if user_result.is_ok() {
            Err(DieselError::DatabaseError(
                DieselDbError::UniqueViolation,
                Box::new("phone number taken".to_string()),
            ))
        } else {
            Ok(true)
        }
    } else {
        Ok(false)
    }
}

pub fn validate_password(password: String) -> bool {
    // TODO: block emoji from password
    let lower_re = Regex::new(LOWERCASE_REGEX).unwrap();
    let upper_re = Regex::new(UPPERCASE_REGEX).unwrap();
    let num_re = Regex::new(NUMERIC_REGEX).unwrap();
    let special_re = Regex::new(SPECIAL_REGEX).unwrap();
    let valid_password = (password.len() >= 12 && password.len() <= 64)
        && (lower_re.is_match(&password))
        && (upper_re.is_match(&password))
        && (num_re.is_match(&password))
        && (special_re.is_match(&password));

    valid_password
}

pub fn validate_bio(bio: &str) -> bool {
    let valid_bio = bio.len() >= 1 && bio.len() <= 500;

    valid_bio
}

pub fn validate_profile_pic(profile_pic: &str) -> bool {
    // remove the "data:image/..." prefix
    let base64_data = if let Some(idx) = profile_pic.find(",") {
        &profile_pic[idx + 1..]
    } else {
        profile_pic
    };

    // decode and load as an image
    BASE64_STANDARD
        .decode(base64_data)
        .ok()
        .and_then(|bytes| load_from_memory(&bytes).ok())
        .is_some()
}
