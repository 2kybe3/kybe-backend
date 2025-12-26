use crate::config::types::Config;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

pub async fn init_db(config: Arc<Config>) -> Result<(), sqlx::Error> {
    let _ = PgPoolOptions::new().max_connections(5).connect(&config.database.postgres_url).await?;

    Ok(())
}
