mod db;
mod model;

use crate::model::{
  TableName,
  League,
  Team,
  Division,
  TeamPlayer,
  Player,
  Shot,
};

use sqlx::{
  Postgres,
  Pool,
};
use axum::{
  Router,
  http::StatusCode,
  extract::{
    Path,
    State,
  },
  response::{
    Json,
    IntoResponse,
  },
  routing::get,
};
use axum_macros::debug_handler;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServerState {
  db_pool: Arc<Pool<Postgres>>,
}

#[tokio::main]
async fn main() {
  let pool = db::connect().await;
  let state = ServerState {
    db_pool: Arc::new(pool),
  }; 
  let router = Router::new()
    .route("/league/", get(league_all))
    .route("/league/:id", get(league_id))
    .route("/division/", get(division_all))
    .route("/division/:id", get(division_id))
    .route("/team/", get(team_all))
    .route("/team/:id", get(team_id))
    .route("/player/", get(player_all))
    .route("/player/:id", get(player_id))
    .route("/team-player/", get(team_player_all))
    .route("/team-playerplayer/:id", get(team_player_id))
    .route("/shot/", get(shots_all))
    .route("/shot/:id", get(shots_id))
    .with_state(state);
  let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
  println!("Listening on {}", addr);
  axum::Server::bind(&addr)
    .serve(router.into_make_service())
    .await
    .unwrap();
}


async fn get_all<T: Send + Unpin + TableName + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>>(pool: &sqlx::PgPool) -> Result<Vec<T>, sqlx::Error> {
  sqlx::query_as::<_, T>(
    &format!("SELECT * FROM {};", <T as TableName>::TABLE_NAME)
  )
  .fetch_all(pool)
  .await
}
async fn get_by_id<T: Send + Unpin + TableName + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>>(pool: &sqlx::PgPool, id: i32) -> Result<Option<T>, sqlx::Error> {
  sqlx::query_as::<_, T>(
    &format!("SELECT * FROM {} WHERE id = $1;", <T as TableName>::TABLE_NAME)
  )
  .bind(id)
  .fetch_optional(pool)
  .await
}
/*
async fn insert_into<T: Sync + Send + Unpin + TableName + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>>(pool: &sqlx::PgPool, new: &T) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
  let query = sql_builder::SqlBuilder::insert_into(<T as TableName>::TABLE_NAME)
    .values(())
    .sql().unwrap();
  sqlx::query(
    &query
  )
  .execute(pool)
  .await
}
*/

macro_rules! get_all {
  ($crud_struct:ident, $func_name:ident) => {
    #[debug_handler]
    async fn $func_name(State(server_config): State<ServerState>) -> impl IntoResponse {
      let cruder = get_all::<$crud_struct>(&server_config.db_pool)
        .await
        .unwrap();
      (StatusCode::OK, Json(cruder))
    }
  }
}
macro_rules! get_by_id {
  ($crud_struct:ident, $func_name:ident) => {
    #[debug_handler]
    async fn $func_name(State(server_config): State<ServerState>, Path(id): Path<i32>) -> impl IntoResponse {
      let cruder = get_by_id::<$crud_struct>(&server_config.db_pool, id)
        .await
        .unwrap();
      (StatusCode::OK, Json(cruder))
    }
  }
}

/*
macro_rules! insert {
  ($crud_struct:ident, $func_name:ident) => {
    #[debug_handler]
    async fn $func_name(State(server_config): State<ServerState>, Json(NewPlayer): Json<NewPlayer>) -> impl IntoResponse {
      let cruder = get_by_id::<$crud_struct>(&server_config.db_pool, id)
        .await
        .unwrap();
      (StatusCode::OK, Json(cruder))
    }
  }
}
*/

macro_rules! impl_all_query_types {
  ($ty:ident, $func_all:ident, $func_by_id:ident) => {
    get_all!($ty, $func_all);
    get_by_id!($ty, $func_by_id);
  }
}

impl_all_query_types!(
  TeamPlayer,
  team_player_all,
  team_player_id
);
impl_all_query_types!(
  Player,
  player_all,
  player_id
);
impl_all_query_types!(
  Team,
  team_all,
  team_id
);
impl_all_query_types!(
  Shot,
  shots_all,
  shots_id
);
impl_all_query_types!(
  Division,
  division_all,
  division_id
);
impl_all_query_types!(
  League,
  league_all,
  league_id
);

