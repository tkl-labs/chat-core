use actix_web::{post, HttpRequest, HttpResponse, Responder};
use chrono::Utc;

#[post("/logout")]
pub async fn post_logout(req: HttpRequest) -> impl Responder {
    println!("{:?}: Logout request from {:?}", Utc::now(), req.peer_addr());
    HttpResponse::Ok().body("Logged out")
}
