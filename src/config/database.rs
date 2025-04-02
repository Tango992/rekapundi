use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};

/// Initializes a connection pool to the PostgreSQL database.
pub async fn init() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not found in env file");

    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tokio::runtime::Runtime;

    // Override printing panic message to the stderr
    fn custom_panic_hook() {
        std::panic::set_hook(Box::new(|_| {}));
    }

    #[test]
    #[serial]
    #[should_panic(expected = "DATABASE_URL not found in env file")]
    fn test_init_missing_env_var() {
        unsafe { env::remove_var("DATABASE_URL") };

        custom_panic_hook();

        let rt = Runtime::new().unwrap();

        rt.block_on(init());
    }

    #[test]
    #[serial]
    #[should_panic(expected = "Failed to connect to the database")]
    fn test_init_invalid_connection() {
        unsafe {
            env::set_var(
                "DATABASE_URL",
                "postgres://invalid:invalid@localhost:5432/nonexistentdb",
            )
        };

        custom_panic_hook();

        let rt = Runtime::new().unwrap();

        rt.block_on(init());

        // Clean up
        unsafe { env::remove_var("DATABASE_URL") };
    }
}
