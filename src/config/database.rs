use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};

// Initializes the PostgreSQL connection pool
pub async fn init() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not found in env file");

    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database")
}
