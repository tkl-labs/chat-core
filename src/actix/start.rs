use super::routes::apply_routes;
use crate::database::init::PGPool;
use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use actix_web::http::header;

const HTTP_SERVER_URL: &str = "127.0.0.1";
const HTTP_SERVER_PORT: u16 = 8080;

pub async fn start_server(pool: PGPool) -> std::io::Result<()> {
    println!(
        "Starting Actix web server on {}:{}",
        HTTP_SERVER_URL, HTTP_SERVER_PORT
    );

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://127.0.0.1:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(cors)
            .configure(apply_routes)
            .app_data(web::Data::new(pool.clone()))
    })
    .bind((HTTP_SERVER_URL, HTTP_SERVER_PORT))?
    .run()
    .await
}
