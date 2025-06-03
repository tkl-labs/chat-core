use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse, Responder, patch, web};
use chrono::Utc;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::actix::auth::jwt::decode_jwt_token;
use crate::database::init::PGPool;
use crate::models::UpdateUser;
use diesel::result::DatabaseErrorKind as DieselDbError;
use diesel::result::Error as DieselError;

#[patch("/profile")]
pub async fn patch_profile(
    pool: web::Data<PGPool>,
    req_body: web::Json<UpdateUser>,
    req: HttpRequest,
) -> impl Responder {
    println!(
        "{:?}: Update profile request from {:?}",
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

    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(value) => value,
        Err(_) => {
            return HttpResponse::Unauthorized()
                .content_type(ContentType::json())
                .body(r#"{"detail":"invalid access token"}"#);
        }
    };

    let data = req_body.into_inner();

    let changes = UpdateUser {
        username: data.username,
        email: data.email,
        phone_number: data.phone_number,
        bio: data.bio,
        profile_pic: data.profile_pic,
    };

    match apply_user_update(pool, user_uuid, changes).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            eprintln!(
                "{:?}: Failed to update user: {:?}",
                Utc::now().timestamp(),
                e
            );
            HttpResponse::BadRequest()
                .content_type(ContentType::json())
                .body(r#"{"detail":"failed to update user"}"#)
        }
    }
}

async fn apply_user_update(
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
