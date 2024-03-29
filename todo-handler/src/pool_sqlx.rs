use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::time::Duration;
use std::error::Error;
use tokio::time::sleep;

pub async fn establish_connection(database_url: &str) -> Result<Pool<Postgres>, sqlx::Error> {
    let max_retries = 5;
    let retry_delay = Duration::from_secs(3);

    for retry_count in 0..=max_retries {
        match PgPoolOptions::new()
            .max_connections(6)
            .min_connections(2)
            .idle_timeout(Some(Duration::from_secs(60)))
            .acquire_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await
        {
            Ok(pool) => return Ok(pool),
            Err(e) if retry_count < max_retries => {
                println!("Error connecting to database: {:?}, retrying... (Attempt {} of {})", e, retry_count + 1, max_retries);
                sleep(retry_delay).await;
            },
            Err(e) => {
                println!("Error connecting to database after {} attempts: {:?}", max_retries, e);
                return Err(e.into());
            }
        }
    }
    unreachable!()
}