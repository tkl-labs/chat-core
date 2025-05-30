use super::routes::apply_routes;
use crate::database::init::PGPool;
use actix_cors::Cors;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use actix_web::{App, HttpResponse, HttpServer, web};
use dotenv::dotenv;
use std::env;

const SERVER_URL: &str = "127.0.0.1";
const HTTP_SERVER_PORT: u16 = 8080;

pub async fn start_server(pool: PGPool) -> std::io::Result<()> {
    dotenv().ok();

    println!(
        "Starting Actix web server on {}:{}",
        SERVER_URL, HTTP_SERVER_PORT
    );

    let session_secret =
        env::var("SESSION_SECRET").expect("ERROR: SESSION_SECRET must be present in '.env'");
    let session_secret_key = Key::from(session_secret.as_bytes());

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_origin("http://127.0.0.1:3000")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![
                        actix_web::http::header::AUTHORIZATION,
                        actix_web::http::header::ACCEPT,
                        actix_web::http::header::CONTENT_TYPE,
                    ])
                    .supports_credentials()
                    .max_age(3600),
            )
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    session_secret_key.clone(),
                )
                .cookie_secure(false)
                .build(),
            )
            .configure(apply_routes)
            .app_data(web::Data::new(pool.clone()))
            .default_service(web::to(|| HttpResponse::Ok()))
    })
    .bind((SERVER_URL, HTTP_SERVER_PORT))?
    .run()
    .await
}
