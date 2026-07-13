use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

pub type DBPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct Database {
    pub pool: DBPool,
}

impl Database {
    pub fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let result: DBPool = r2d2::Pool::builder()
            .max_size(10)
            .min_idle(Some(2))
            .connection_timeout(std::time::Duration::from_secs(10))
            .build(manager)
            .expect("Failed to create pool.");
        Database { pool: result }
    }
}
