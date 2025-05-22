use backend::actix;
use backend::database::init::init_pool;

use actix_web;

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
            println!("connection pool created");
            pool
        }
    };

    let x = actix::start_server(pool).await;
    return x;
}
