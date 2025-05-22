mod login;
use login::{get_login, post_login};

mod register;
use register::{get_register, post_register};

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_login).service(post_login);
    cfg.service(get_register).service(post_register);
}
