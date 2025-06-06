use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub phone_number: String,
    pub two_factor_auth: bool,
    pub password_hash: String,
    pub profile_pic: Option<String>,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::friendships)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Friendship {
    pub user_id: Uuid,
    pub friend_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub friendship_status: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RegisterUser {
    pub username: String,
    pub email: String,
    pub phone_number: String,
    pub password_hash: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LoginUser {
    pub username: String,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub bio: Option<String>,
    pub profile_pic: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::friendships)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateFriendship {
    pub user_id: Uuid,
    pub friend_id: Uuid,
    pub friendship_status: String,
}