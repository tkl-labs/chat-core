use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use chrono::Utc;
use diesel::{ExpressionMethods, query_dsl::methods::FilterDsl};
use diesel_async::RunQueryDsl;
use serde_json::to_string;
use std::collections::HashMap;
use uuid::Uuid;

use crate::actix::auth::jwt::decode_jwt_token;
use crate::database::init::PGPool;
use crate::models::User;
use diesel::result::DatabaseErrorKind as DieselDbError;
use diesel::result::Error as DieselError;

#[get("/me")]
pub async fn get_me(pool: web::Data<PGPool>, req: HttpRequest) -> impl Responder {
    println!(
        "{:?}: Me request from {:?}",
        Utc::now().timestamp() as usize,
        req.peer_addr()
    );

    // extract access token from cookie
    let access_token = match req.cookie("access_token") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"missing jwt token"}"#);
        }
    };

    // decode and validate JWT token
    let claim = match decode_jwt_token(access_token) {
        Ok(claim) => claim,
        Err(_) => {
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid access token"}"#);
        }
    };

    let user_id = claim.sub;

    match get_user_by_id(pool, &user_id).await {
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

            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(json_str)
        }
        Err(e) => {
            eprintln!(
                "{:?}: User fetching failed: {:?}",
                Utc::now().timestamp() as usize,
                e
            );

            HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"User not found"}"#)
        }
    }
}

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
