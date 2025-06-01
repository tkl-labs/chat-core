mod csrf;
use csrf::get_csrf;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_csrf);
}
