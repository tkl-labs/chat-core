use crate::services::csrf::generate_csrf_token;
use actix_web::{HttpRequest, HttpResponse, Responder, get};
use chrono::Utc;
use serde_json::to_string;
use std::collections::HashMap;

#[get("/csrf")]
pub async fn get_csrf(req: HttpRequest) -> impl Responder {
    println!(
        "{:?}: Generated and sent CSRF token to {:?}",
        Utc::now().timestamp() as usize,
        req.peer_addr()
    );

    let token = generate_csrf_token();

    let mut map = HashMap::new();
    map.insert("csrf_token", token);

    let json_str = to_string(&map).unwrap();

    HttpResponse::Ok().body(json_str)
}
