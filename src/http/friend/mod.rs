pub mod friendship;
use friendship::{delete_remove, get_all, patch_add, post_add, get_friend_requests};

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(delete_remove)
        .service(get_all)
        .service(patch_add)
        .service(post_add)
        .service(get_friend_requests);
}
