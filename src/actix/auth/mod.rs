mod example;
use example::get_example;

mod login;
use login::post_login;

mod logout;
use logout::post_logout;

mod register;
use register::post_register;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_example)
        .service(post_login)
        .service(post_logout)
        .service(post_register);
}
