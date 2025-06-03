mod login;
use login::post_login;

mod logout;
use logout::post_logout;

mod register;
use register::post_register;

pub mod jwt;
use jwt::post_refresh;

mod me;
use me::get_me;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(post_login)
        .service(post_logout)
        .service(post_register)
        .service(get_me)
        .service(post_refresh);
}
