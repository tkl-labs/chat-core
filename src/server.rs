use backend::db::operations::init_pool;

use chrono::Utc;
use std::io::{Error, ErrorKind};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let result = init_pool(5).await;

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

    backend::http::start_http_server(pool).await
}