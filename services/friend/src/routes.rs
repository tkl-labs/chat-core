use crate::friend::{get_all, get_friend_requests, patch_add, post_add, post_remove};
use actix_web::web;

static SCOPE_HANDLERS: &[(&str, fn(&mut web::ServiceConfig))] = &[("friend", routes)];

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all)
        .service(get_friend_requests)
        .service(patch_add)
        .service(post_add)
        .service(post_remove);
}

pub fn apply_routes(cfg: &mut web::ServiceConfig) {
    for (path, handlers_appliers) in SCOPE_HANDLERS {
        cfg.service(web::scope(path).configure(handlers_appliers));
    }
}
