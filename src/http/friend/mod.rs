pub mod friendship;
use friendship::{delete_remove, get_all, post_add};

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(delete_remove)
        .service(get_all)
        .service(post_add);
}
