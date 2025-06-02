use actix_web::{HttpRequest, HttpResponse, Responder, get};
use chrono::Utc;
use serde_json::to_string;
use std::collections::HashMap;

#[get("/csrf")]
pub async fn get_csrf(req: HttpRequest) -> impl Responder {
    println!(
        "{:?}: Generated and sent CSRF token to {:?}",
        Utc::now(),
        req.peer_addr()
    );

    let token = generate_csrf_token();

    let mut map = HashMap::new();
    map.insert("csrf_token", token);

    let json_str = to_string(&map).unwrap();

    return HttpResponse::Ok().body(json_str);
}

use csrf::{AesGcmCsrfProtection, CsrfProtection};

fn generate_csrf_token() -> String {
    let protect = AesGcmCsrfProtection::from_key(*b"01234567012345670123456701234567");

    let (token, _) = protect
        .generate_token_pair(None, 300)
        .expect("couldn't generate token/cookie pair");

    let token_str = token.b64_string();

    return token_str;
}

pub fn verify_csrf_token(req: &HttpRequest) -> bool {
    match req.headers().get("X-CSRF-Token") {
        Some(_) => return true,
        None => return false,
    };
}
