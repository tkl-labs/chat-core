pub mod csrf;
use csrf::get_csrf;

pub mod jwt;
use jwt::post_refresh;

pub mod login;
use login::post_login;

pub mod logout;
use logout::post_logout;

pub mod register;
use register::post_register;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_csrf)
        .service(post_login)
        .service(post_logout)
        .service(post_refresh)
        .service(post_register);
}
