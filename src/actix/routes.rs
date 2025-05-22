use super::auth::routes as auth_routes;
use actix_web::web;

static AUTH_SCOPE_HANDLERS: &[(&str, fn(&mut web::ServiceConfig))] = &[("auth", auth_routes)];

pub fn apply_routes(cfg: &mut web::ServiceConfig) {
    for (path, handlers_appliers) in AUTH_SCOPE_HANDLERS {
        cfg.service(web::scope(path).configure(handlers_appliers));
    }
}
