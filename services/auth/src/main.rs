mod auth;
mod csrf;
mod jwt;
mod login;
mod logout;
mod register;
mod routes;

use crate::routes::apply_routes;
use shared::database::create_database_pool;

use actix_cors::Cors;
use actix_web::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderName};
use actix_web::{App, HttpServer, web};
use chrono::Utc;
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use std::io::{Error, ErrorKind, Result};

const SERVER_URL: &str = "0.0.0.0";
const HTTP_SERVER_PORT: u16 = 8080;

#[actix_web::main]
async fn main() -> Result<()> {
    // Initialize OTLP exporter using gRPC (Tonic)
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://jaeger:4317")
        .build()
        .expect("Failed to initialise OLTP exporter");

    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_resource(
            Resource::builder()
                .with_service_name("auth-service")
                .build(),
        )
        .build();

    // Set it as the global provider
    global::set_tracer_provider(provider);

    // Initialise database connection pool
    let result = create_database_pool(5).await;

    let pool = match result {
        Err(e) => {
            eprintln!("{}", e);
            return Err(Error::new(ErrorKind::Other, e));
        }
        Ok(pool) => {
            println!(
                "{:?}: Connection pool created",
                Utc::now().timestamp() as usize
            );
            pool
        }
    };

    // Run the HTTP server until application close
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
                    .allowed_origin("tauri://localhost") // macOS build
                    .allowed_origin("http://tauri.localhost") // Windows build
                    .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
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
