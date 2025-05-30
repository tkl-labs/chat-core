use actix_session::Session;
use actix_web::{HttpResponse, Responder, post};

#[post("/logout")]
pub async fn post_logout(session: Session) -> impl Responder {
    session.purge();
    HttpResponse::Ok().body("Logged out")
}
