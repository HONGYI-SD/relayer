use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use postgres::{Client, NoTls};

use crate::common::node_configs::StoreConfiguration;

pub type PgConnectionPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type PooledPgConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub fn create_pool(config: StoreConfiguration, pool_size: u32) -> PgConnectionPool {
    let config = config.to_owned();

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        config.username,
        config.password,
        config.host,
        config.port,
        config.schema,
    );

    PgConnectionPool::builder()
        .max_size(pool_size)
        .build(ConnectionManager::<PgConnection>::new(database_url))
        .expect("Connection pool to postgres cannot be created")
}


pub fn create_one(config: StoreConfiguration) -> Client {
    let config = config.to_owned();

    let connection_str = format!(
        "host={} user={} password={} dbname={} port={}",
        config.host,
        config.username,
        config.password,
        config.schema,
        config.port,
    );

    Client::connect(&connection_str, NoTls).expect(format!("the config is {}", connection_str).as_str())
}

