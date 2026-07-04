use anyhow::Result;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::PgPool;
use sqlx::SqlitePool;

pub enum DatabasePool {
    Sqlite(SqlitePool),
    Postgres(PgPool),
}

impl DatabasePool {
    pub async fn connect_sqlite(path: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(path)
            .await?;
        Ok(Self::Sqlite(pool))
    }

    pub async fn connect_postgres(connection_string: &str) -> Result<Self> {
        let pool = PgPool::connect(connection_string).await?;
        Ok(Self::Postgres(pool))
    }
}
