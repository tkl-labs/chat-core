use crate::csrf::get_csrf;
use crate::jwt::post_refresh;
use crate::login::post_login;
use crate::logout::post_logout;
use crate::register::post_register;
use actix_web::web;

static SCOPE_HANDLERS: &[(&str, fn(&mut web::ServiceConfig))] = &[("auth", routes)];

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_csrf)
        .service(post_refresh)
        .service(post_login)
        .service(post_logout)
        .service(post_register);
}

pub fn apply_routes(cfg: &mut web::ServiceConfig) {
    for (path, handlers_appliers) in SCOPE_HANDLERS {
        cfg.service(web::scope(path).configure(handlers_appliers));
    }
}
