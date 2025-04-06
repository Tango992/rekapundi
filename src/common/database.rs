use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing;

/// Initializes a connection pool to the PostgreSQL database.
pub async fn init() -> Result<PgPool, Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL").inspect_err(|_| {
        tracing::error!("DATABASE_URL not found in environment");
    })?;

    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .inspect_err(|_| {
            tracing::error!("Failed to connect to the database");
        })?;

    Ok(pg_pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tokio::runtime::Runtime;

    #[test]
    #[serial]
    fn test_init_missing_env_var() {
        unsafe { env::remove_var("DATABASE_URL") };

        let rt = Runtime::new().unwrap();

        let result = rt.block_on(init());
        assert!(result.is_err_and(|e| e.downcast_ref() == Some(&std::env::VarError::NotPresent)));
    }

    #[test]
    #[serial]
    fn test_init_invalid_connection() {
        unsafe {
            env::set_var(
                "DATABASE_URL",
                "postgres://invalid:invalid@localhost:5432/nonexistentdb",
            )
        };

        let rt = Runtime::new().unwrap();
        // Use a different database URL that is invalid
        let result = rt.block_on(init());

        assert!(result.is_err());

        // Clean up
        unsafe { env::remove_var("DATABASE_URL") };
    }
}
