use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::{BuildError, Object, Pool};
use dotenv::dotenv;
use std::{env, usize};

pub type PGPool = Pool<AsyncPgConnection>;

pub async fn init_pool(max_size: usize) -> Result<PGPool, BuildError> {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("ERROR: DATABASE_URL must be present in '.env'");

    let pool_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
    let pool_result = Pool::builder(pool_config).max_size(max_size).build();

    let pool: PGPool = match pool_result {
        Err(e) => return Err(e),
        Ok(value) => value,
    };

    let mut connections = Vec::<Object<AsyncPgConnection>>::new();

    for _ in 0..max_size {
        let connection = pool.get().await;

        match connection {
            Err(e) => panic!("{}", e),
            Ok(con) => connections.push(con),
        };
    }

    drop(connections);

    return Ok(pool);
}
