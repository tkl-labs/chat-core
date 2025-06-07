pub mod friend;
use friend::{post_remove, get_all, get_friend_requests, patch_add, post_add};

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(post_remove)
        .service(get_all)
        .service(patch_add)
        .service(post_add)
        .service(get_friend_requests);
}
