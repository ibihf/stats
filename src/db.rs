use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub async fn connect() -> Pool<Postgres> {
    PgPoolOptions::new()
        .max_connections(8)
        .connect("postgres://ibihf:ibihf@localhost/ibihf")
        .await
        .unwrap()
}
