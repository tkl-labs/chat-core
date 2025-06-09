use actix_web::HttpRequest;
use csrf::{AesGcmCsrfProtection, CsrfProtection};

pub fn generate_csrf_token() -> String {
    let protect = AesGcmCsrfProtection::from_key(*b"01234567012345670123456701234567");

    let (token, _) = protect
        .generate_token_pair(None, 300)
        .expect("couldn't generate token/cookie pair");

    let token_str = token.b64_string();

    token_str
}

pub fn verify_csrf_token(req: &HttpRequest) -> bool {
    match req.headers().get("X-CSRF-Token") {
        Some(_) => true,
        None => false,
    }
}
