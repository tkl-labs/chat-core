use actix_session::Session;
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, Responder, post, web};
use diesel::dsl::insert_into;
use diesel_async::RunQueryDsl;
use regex::Regex;
use serde::Deserialize;

use crate::database::init::PGPool;
use diesel::result::DatabaseErrorKind as DieselDbError;
use diesel::result::Error as DieselError;

#[derive(Deserialize)]
struct RegisterForm {
    username: String,
    email: String,
    phone_number: String,
    password: String,
}

const LOWERCASE_REGEX: &str = "[a-z]";
const UPPERCASE_REGEX: &str = "[A-Z]";
const NUMERIC_REGEX: &str = "[0-9]";
const SPECIAL_REGEX: &str = "[^a-zA-Z0-9]";
const EMAIL_REGEX: &str = r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$";
const PHONE_NUMBER_REGEX: &str = r"^\+?[0-9]{7,15}$";

#[post("/register")]
pub async fn post_register(
    pool: web::Data<PGPool>,
    req_body: web::Json<RegisterForm>,
    session: Session,
) -> impl Responder {
    let username = &req_body.username.trim();
    let email = &req_body.email.trim();
    let phone_number = &req_body.phone_number.trim();
    let password = &req_body.password.trim();

    // sanitise username
    let username_meets_requirements = (username.len() >= 8 && username.len() <= 16)
        && (username.chars().all(char::is_alphanumeric));

    if !username_meets_requirements {
        return HttpResponse::BadRequest()
            .content_type(ContentType::json())
            .body(r#"{"detail":"invalid username"}"#);
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
        return HttpResponse::BadRequest()
            .content_type(ContentType::json())
            .body(r#"{"detail":"invalid password"}"#);
    }

    // sanitise email
    let email_re = Regex::new(EMAIL_REGEX).unwrap();
    let email_meets_requirements = email_re.is_match(&email);
    if !email_meets_requirements {
        return HttpResponse::BadRequest()
            .content_type(ContentType::json())
            .body(r#"{"detail":"invalid email"}"#);
    }

    // sanitise phone number
    let phone_re = Regex::new(PHONE_NUMBER_REGEX).unwrap();
    let phone_number_meets_requirements = phone_re.is_match(&phone_number);
    if !phone_number_meets_requirements {
        return HttpResponse::BadRequest()
            .content_type(ContentType::json())
            .body(r#"{"detail":"invalid phone number"}"#);
    }

    // create a hash of user password
    let password_hash = match bcrypt::hash(password, 10) {
        Ok(password_hash) => password_hash,
        Err(e) => {
            eprintln!("Internal server error: {}", e);
            return HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(r#"{"detail":"an unexpected error occurred"}"#);
        }
    };

    // attempt to insert a new user into db
    match add_user_to_db(pool, username, email, phone_number, &password_hash).await {
        Ok(_) => {
            // store username_or_email in the session cookie
            if let Err(e) = session.insert("username_or_email", username) {
                eprintln!("Internal server error: {}", e);
                return HttpResponse::InternalServerError()
                    .content_type(ContentType::json())
                    .body(r#"{"detail":"an unexpected error occurred"}"#);
            }

            return HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(r#"{"detail":"account created successfully"}"#);
        }
        Err(DieselError::DatabaseError(DieselDbError::UniqueViolation, _)) => {
            return HttpResponse::Conflict()
                .content_type(ContentType::json())
                .body(r#"{"detail":"an account with this email or username already exists"}"#);
        }
        Err(e) => {
            eprintln!("Internal server error: {}", e);
            return HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(r#"{"detail":"an unexpected error occurred"}"#);
        }
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
        eprintln!("Failed to acquire DB connection: {:?}", e);
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

    let result = insert_into(users::table)
        .values(&new_user)
        .execute(&mut conn)
        .await?;

    Ok(result)
}
