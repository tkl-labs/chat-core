pub mod profile;
use profile::{get_me, get_profile, patch_profile};

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_me)
        .service(get_profile)
        .service(patch_profile);
}
