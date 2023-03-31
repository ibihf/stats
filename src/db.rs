use sqlx::{Postgres, Pool};
use sqlx::postgres::PgPoolOptions;

pub async fn connect() -> Pool<Postgres> {
  PgPoolOptions::new()
    .max_connections(8)
    .connect("postgres://ibihf:ibihf@localhost/ibihf").await
    .unwrap()
}
