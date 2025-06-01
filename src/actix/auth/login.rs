use actix_session::Session;
use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse, Responder, post, web};
use diesel::{ExpressionMethods, query_dsl::methods::FilterDsl};
use diesel_async::RunQueryDsl;
use regex::Regex;
use serde::Deserialize;

use crate::actix::api::verify_csrf_token;
use crate::database::init::PGPool;
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
    session: Session,
    req: HttpRequest,
) -> impl Responder {
    let verify = verify_csrf_token(&req);

    if !verify {
        return HttpResponse::Unauthorized().body("CSRF fail");
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
        Ok(result) => {
            if result == true {
                // store username in the session cookie
                if let Err(e) = session.insert("username", username) {
                    eprintln!("Session insert error: {}", e);
                    return HttpResponse::InternalServerError()
                        .content_type(ContentType::json())
                        .body(r#"{"detail":"an unexpected error occurred"}"#);
                }

                return HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(r#"{"detail":"logged in successfully"}"#);
            } else {
                return HttpResponse::Unauthorized()
                    .content_type(ContentType::json())
                    .body(r#"{"detail":"invalid login"}"#);
            }
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(r#"{"detail":"an unexpected error occurred"}"#);
        }
    }
}

pub async fn check_user_in_db(
    pool: web::Data<PGPool>,
    uname: &str,
    pass: &str,
) -> Result<bool, DieselError> {
    use crate::models::User;
    use crate::schema::users::dsl::*;

    let mut conn = pool.get().await.map_err(|e| {
        eprintln!("Failed to acquire DB connection: {:?}", e);
        DieselError::DatabaseError(DieselDbError::UnableToSendCommand, Box::new(e.to_string()))
    })?;

    let result = users
        .filter(username.eq(uname))
        .first::<User>(&mut conn)
        .await;

    match result {
        Ok(user) => match bcrypt::verify(pass, &user.password_hash) {
            Ok(_) => Ok(true),
            Err(e) => {
                eprintln!("Password verification failed: {:?}", e);
                Ok(false)
            }
        },
        Err(DieselError::NotFound) => Ok(false),
        Err(e) => {
            eprintln!("Database error during login: {:?}", e);
            Err(e)
        }
    }
}
