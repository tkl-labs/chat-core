mod csrf;
use csrf::get_csrf;
pub use csrf::verify_csrf_token;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_csrf);
}

