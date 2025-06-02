use super::routes::apply_routes;
use crate::database::init::PGPool;
use actix_cors::Cors;
use actix_web::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderName};
use actix_web::{App, HttpServer, web};
use chrono::Utc;

const SERVER_URL: &str = "127.0.0.1";
const HTTP_SERVER_PORT: u16 = 8080;

pub async fn start_server(pool: PGPool) -> std::io::Result<()> {
    println!(
        "{:?}: Starting Actix web server on {:?}:{:?}",
        Utc::now().timestamp() as usize,
        SERVER_URL,
        HTTP_SERVER_PORT
    );

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_origin("http://127.0.0.1:3000")
                    .allowed_origin("http://tauri.localhost")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![
                        AUTHORIZATION,
                        ACCEPT,
                        CONTENT_TYPE,
                        HeaderName::from_static("x-csrf-token"),
                    ])
                    .supports_credentials()
                    .max_age(3600),
            )
            .configure(apply_routes)
            .app_data(web::Data::new(pool.clone()))
    })
    .bind((SERVER_URL, HTTP_SERVER_PORT))?
    .run()
    .await
}
