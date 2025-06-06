use super::auth::routes as auth_routes;
use super::friend::routes as friend_routes;
use super::profile::routes as profile_routes;
use actix_web::web;

static SCOPE_HANDLERS: &[(&str, fn(&mut web::ServiceConfig))] = &[
    ("auth", auth_routes),
    ("friend", friend_routes),
    ("profile", profile_routes),
];

pub fn apply_routes(cfg: &mut web::ServiceConfig) {
    for (path, handlers_appliers) in SCOPE_HANDLERS {
        cfg.service(web::scope(path).configure(handlers_appliers));
    }
}
