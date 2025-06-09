use crate::profile::{get_profile, patch_profile};
use actix_web::web;

static SCOPE_HANDLERS: &[(&str, fn(&mut web::ServiceConfig))] = &[("profile", routes)];

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_profile)
        .service(patch_profile);
}

pub fn apply_routes(cfg: &mut web::ServiceConfig) {
    for (path, handlers_appliers) in SCOPE_HANDLERS {
        cfg.service(web::scope(path).configure(handlers_appliers));
    }
}
