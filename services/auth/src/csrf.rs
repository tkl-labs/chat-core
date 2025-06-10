use actix_web::{HttpRequest, HttpResponse, Responder, get};
use chrono::Utc;
use opentelemetry::{
    global,
    KeyValue,
    trace::{Span, Tracer},
};
use serde_json::to_string;
use shared::csrf::generate_csrf_token;
use std::collections::HashMap;

#[get("/csrf")]
pub async fn get_csrf(req: HttpRequest) -> impl Responder {
    let tracer = global::tracer("my_tracer");

    let mut span = tracer.start("get_csrf");
    span.set_attribute(KeyValue::new("rpc.method", "get_csrf"));
    
    println!(
        "{:?}: GET /auth/csrf from {:?}",
        Utc::now().timestamp() as usize,
        req.peer_addr()
    );

    let token = generate_csrf_token();

    let mut map = HashMap::new();
    map.insert("csrf_token", token);

    let json_str = to_string(&map).unwrap();

    span.end();
    HttpResponse::Ok().body(json_str)
}
