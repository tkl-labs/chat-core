use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::{BuildError, Pool};
use dotenv::dotenv;
use std::{env, usize};

pub type PGPool = Pool<AsyncPgConnection>;

pub async fn init_pool(max_size: usize) -> Result<PGPool, BuildError> {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("ERROR: DATABASE_URL must be present in '.env'");

    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);

    let pool = Pool::builder(config).max_size(max_size).build()?;

    Ok(pool)
}
