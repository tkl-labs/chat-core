use actix_web::{HttpRequest, HttpResponse, Responder, get};
use serde_json::to_string;
use std::collections::HashMap;

#[get("/csrf")]
pub async fn get_csrf() -> impl Responder {
    let (token, cookie) = generate_csrf_token();

    let mut map = HashMap::new();
    map.insert("csrf_token", token);
    map.insert("csrf_cookie", cookie);

    let json_str = to_string(&map).unwrap();

    return HttpResponse::Ok().body(json_str);
}

use base64::prelude::*;
use csrf::{AesGcmCsrfProtection, CsrfProtection};

fn generate_csrf_token() -> (String, String) {
    let protect = AesGcmCsrfProtection::from_key(*b"01234567012345670123456701234567");

    let (token, cookie) = protect
        .generate_token_pair(None, 300)
        .expect("couldn't generate token/cookie pair");

    let token_str = token.b64_string();
    let cookie_str = cookie.b64_string();

    return (token_str, cookie_str);
}

pub fn verify_csrf_token(req: &HttpRequest) -> bool {
    let token_str = req
        .headers()
        .get("X-CSRF-Token")
        .and_then(|val| val.to_str().ok())
        .map(|s| s.to_string())
        .expect("could not convert x-csrf-token to string");
    let cookie_str = req
        .cookie("csrf_token")
        .map(|cookie| cookie.value().to_string())
        .expect("could not convert csrf_token to string");

    let token_bytes = match BASE64_STANDARD.decode(token_str.as_bytes()) {
        Ok(bytes) => bytes,
        Err(_) => return false, // token not base64
    };

    let cookie_bytes = match BASE64_STANDARD.decode(cookie_str.as_bytes()) {
        Ok(bytes) => bytes,
        Err(_) => return false, // cookie not base64
    };

    let protect = AesGcmCsrfProtection::from_key(*b"01234567012345670123456701234567");

    let parsed_token = match protect.parse_token(&token_bytes) {
        Ok(token) => token,
        Err(_) => return false, // token not parsed
    };

    let parsed_cookie = match protect.parse_cookie(&cookie_bytes) {
        Ok(cookie) => cookie,
        Err(_) => return false, // cookie not parsed
    };

    protect
        .verify_token_pair(&parsed_token, &parsed_cookie)
        .is_ok()
}
