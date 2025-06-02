use actix_web::{HttpRequest, HttpResponse, Responder, post};
use chrono::Utc;

#[post("/logout")]
pub async fn post_logout(req: HttpRequest) -> impl Responder {
    println!(
        "{:?}: Logout request from {:?}",
        Utc::now().timestamp() as usize,
        req.peer_addr()
    );
    HttpResponse::Ok().body("Logged out")
}
