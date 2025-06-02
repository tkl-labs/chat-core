use std::collections::HashMap;

use actix_web::cookie::*;
use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse, Responder, post, web};
use bcrypt;
use chrono::Utc;
use diesel::{ExpressionMethods, query_dsl::methods::FilterDsl};
use diesel_async::RunQueryDsl;
use regex::Regex;
use serde::Deserialize;
use serde_json::to_string;

use crate::actix::api::verify_csrf_token;
use crate::actix::auth::jwt::encode_jwt_token;
use crate::database::init::PGPool;
use crate::models::User;
use diesel::result::DatabaseErrorKind as DieselDbError;
use diesel::result::Error as DieselError;

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

const LOWERCASE_REGEX: &str = "[a-z]";
const UPPERCASE_REGEX: &str = "[A-Z]";
const NUMERIC_REGEX: &str = "[0-9]";
const SPECIAL_REGEX: &str = "[^a-zA-Z0-9]";

#[post("/login")]
pub async fn post_login(
    pool: web::Data<PGPool>,
    req_body: web::Json<LoginForm>,
    req: HttpRequest,
) -> impl Responder {
    println!("{:?}: Login request from {:?}", Utc::now(), req.peer_addr());

    let verify = verify_csrf_token(&req);

    if !verify {
        return HttpResponse::Unauthorized()
            .content_type(ContentType::json())
            .body(r#"{"detail":"csrf failed"}"#);
    }

    let username = &req_body.username.trim();
    let password = &req_body.password.trim();

    // sanitise username
    let username_meets_requirements = (username.len() >= 8 && username.len() <= 16)
        && (username.chars().all(char::is_alphanumeric));

    if !username_meets_requirements {
        return HttpResponse::Unauthorized()
            .content_type(ContentType::json())
            .body(r#"{"detail":"invalid login"}"#);
    }

    // sanitise password
    // TODO: block emoji from password
    let lower_re = Regex::new(LOWERCASE_REGEX).unwrap();
    let upper_re = Regex::new(UPPERCASE_REGEX).unwrap();
    let num_re = Regex::new(NUMERIC_REGEX).unwrap();
    let special_re = Regex::new(SPECIAL_REGEX).unwrap();
    let password_meets_requirements = (password.len() >= 12 && password.len() <= 64)
        && (lower_re.is_match(&password))
        && (upper_re.is_match(&password))
        && (num_re.is_match(&password))
        && (special_re.is_match(&password));
    if !password_meets_requirements {
        return HttpResponse::Unauthorized()
            .content_type(ContentType::json())
            .body(r#"{"detail":"invalid login"}"#);
    }

    // attempt to insert a new user into db
    match check_user_in_db(pool, username, password).await {
        Ok(user) => {
            let mut map = HashMap::new();
            map.insert("id", user.id.to_string());
            map.insert("username", user.username);
            map.insert("email", user.email);
            map.insert("phone_number", user.phone_number);
            map.insert("two_factor_auth", user.two_factor_auth.to_string());
            map.insert("profile_pic", user.profile_pic.unwrap_or("".to_string()));
            map.insert("bio", user.bio.unwrap_or("".to_string()));
            map.insert("created_at", user.created_at.to_string());

            let json_str = to_string(&map).unwrap();
            let jwt_token = encode_jwt_token(user.id.to_string());
            let mut jwt_cookie = Cookie::build("jwt_token", "")
                .secure(false) // re-enable for HTTPS
                .http_only(true)
                .max_age(time::Duration::minutes(15))
                .finish();

            match jwt_token {
                Ok(val) => {
                    jwt_cookie = Cookie::build("jwt_token", val)
                        .secure(false) // for localhost, enable secure for HTTPS in prod
                        .http_only(true)
                        .max_age(time::Duration::minutes(15))
                        .same_site(SameSite::Lax)
                        .path("/")
                        .domain("127.0.0.1")
                        .finish()
                }
                Err(_) => {}
            }

            HttpResponse::Ok()
                .content_type(ContentType::json())
                .cookie(jwt_cookie)
                .body(json_str)
        }
        Err(e) => {
            eprintln!("{:?}: Login failed: {:?}", Utc::now(), e);
            HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"login failed"}"#)
        }
    }
}

pub async fn check_user_in_db(
    pool: web::Data<PGPool>,
    uname: &str,
    pass: &str,
) -> Result<User, DieselError> {
    use crate::models::User;
    use crate::schema::users::dsl::*;

    let mut conn = pool.get().await.map_err(|e| {
        eprintln!("{:?}: Failed to acquire DB connection: {:?}", Utc::now(), e);
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
