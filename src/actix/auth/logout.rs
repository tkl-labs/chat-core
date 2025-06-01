use actix_session::Session;
use actix_web::{post, HttpRequest, HttpResponse, Responder};
use chrono::Utc;

#[post("/logout")]
pub async fn post_logout(session: Session, req: HttpRequest) -> impl Responder {
    println!("{:?}: Logout request from {:?}", Utc::now(), req.peer_addr());
    
    session.purge();
    HttpResponse::Ok().body("Logged out")
}
