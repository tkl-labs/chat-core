use actix_session::Session;
use actix_web::{HttpResponse, Responder, get};

#[get("/example")]
pub async fn get_example(session: Session) -> impl Responder {
    // get user session
    if let Some(username_or_email) = session.get::<String>("username_or_email").unwrap() {
        HttpResponse::Ok().json(username_or_email)
    } else {
        HttpResponse::Unauthorized().finish()
    }
}
