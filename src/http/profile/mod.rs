pub mod profile;
use profile::{get_profile, patch_profile};

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_profile)
        .service(patch_profile);
}
